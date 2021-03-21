use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::bindgroups::lighting::LightBindGroup;
use crate::renderer::bindgroups::shadow::ShadowBindGroup;
use crate::renderer::bindgroups::uniforms::UniformBindGroup;
use crate::renderer::bindgroups::HorizonBindGroup;
use crate::renderer::primitives::lights::directionallight::DirectionalLight;
use crate::renderer::primitives::lights::directionallight::DirectionalLightRaw;
use crate::renderer::primitives::lights::pointlight::PointLightRaw;
use crate::renderer::primitives::lights::spotlight::SpotLightRaw;

use crate::{
    components::transform,
    renderer::primitives::{texture::Texture, vertex::Vertex},
    systems::physics::{Physics, PhysicsWorld},
};
use crate::{filesystem::modelimporter::Importer, renderer::model::HorizonModel};
use crate::{renderer::cam::Camera, systems::movement};
use std::{
    num::NonZeroU32,
    ops::{Add, Deref},
};

use super::{
    model::DrawModel,
    primitives::{
        instance::{Instance, InstanceRaw},
        uniforms::{Globals, ShadowUniforms},
        vertex::ModelVertex,
    },
};
use crate::components::physicshandle::*;
use crate::components::transform::*;
use crate::renderer::modelbuilder::ModelBuilder;
use crate::renderer::pass::Pass;
use crate::renderer::utils::texturerenderer::TextureRenderer;
use bytemuck::{bytes_of, cast_slice};
use chrono::{Duration, DurationRound, Timelike};

use glm::{identity, quat_angle_axis, Mat3, Vec3};

use nalgebra::Isometry3;
use rapier3d::{
    dynamics::RigidBodyBuilder,
    geometry::{ColliderBuilder, TriMesh},
};
use specs::{Builder, Join, RunNow, World, WorldExt};
use wgpu::{
    util::DeviceExt, BindGroup, BufferBindingType, DepthStencilState, ShaderFlags, ShaderStage,
};
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    pub queue: wgpu::Queue,
    sc_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
}

const NUM_INSTANCES_PER_ROW: u32 = 15;

impl State {
    //TODO: Move this to a constants / limits struct for cleanliness
    pub const OPENGL_TO_WGPU_MATRIX: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ];
    pub const MAX_ENTITY_COUNT: wgpu::BufferAddress = (u16::MAX / 4) as wgpu::BufferAddress;
    pub const MAX_POINT_LIGHTS: usize = 32;
    pub const MAX_SPOT_LIGHTS: usize = 32;
    pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        depth_or_array_layers: Self::MAX_POINT_LIGHTS as u32,
        height: 1024,
        width: 1024,
    };
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: Some("Device descriptor"),
                },
                None,
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
            format: if cfg!(target_arch = "wasm32") {
                wgpu::TextureFormat::Bgra8Unorm
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            },
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // ! Method should END HERE
        // Lights
        //TODO: add spot / point lights
        DirectionalLight::new(
            glm::vec3(-100000.0, 500000.0, 100000.0),
            wgpu::Color {
                r: 1.0,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
        );

        let shadow_pass = {
            let vs_module =
                device.create_shader_module(&wgpu::include_spirv!("../shaders/shadow.vert.spv"));
            let depth_stencil_state = wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                bias: wgpu::DepthBiasState {
                    clamp: 0.0,
                    constant: 2, // bilinear filtering
                    slope_scale: 2.0,
                },
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                stencil: wgpu::StencilState::default(),
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            };
            let shadow_pipeline = Self::create_pipeline(
                None,
                &vs_module,
                &device,
                &pipeline_layout,
                &[ModelVertex::desc()],
                Some("Shadow pipeline"),
                Some(depth_stencil_state),
            );
            Pass {
                bind_group,
                pipeline: shadow_pipeline,
                uniform_buffer,
            }
        };
        // SHADER

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &diffuse_bind_group_layout,
                    &uniform_bind_group_layout,
                    &light_bind_group_layout,
                ],
                label: Some("Render pipeline layout"),
                push_constant_ranges: &[],
            });

        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../shaders/shader.frag.spv")),
            flags: ShaderFlags::empty(),
            label: Some("forward fragment shader"),
        });
        let target = &[sc_desc.format.into()];
        let val = Some(wgpu::FragmentState {
            targets: target,
            module: &fs_module,
            entry_point: "main",
        });

        let depth_stencil_state = wgpu::DepthStencilState {
            bias: wgpu::DepthBiasState {
                ..Default::default()
            },
            clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            depth_compare: wgpu::CompareFunction::Less,
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        };

        let basic_pipeline = State::create_pipeline(
            val,
            &vs_module,
            &device,
            &render_pipeline_layout,
            &[ModelVertex::desc()],
            Some("Render pipeline"),
            Some(depth_stencil_state),
        );
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline"),
                bind_group_layouts: &[&uniform_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            // https://github.com/gfx-rs/naga/issues/406 have to disable validation
            let light_vs = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                source: wgpu::util::make_spirv(include_bytes!("../shaders/light.vert.spv")),
                flags: ShaderFlags::empty(),
                label: Some("light_vertex_shader"),
            });
            let light_fs =
                device.create_shader_module(&wgpu::include_spirv!("../shaders/light.frag.spv"));
            let target = &[sc_desc.format.into()];
            let fragment_state = Some(wgpu::FragmentState {
                targets: target,
                module: &light_fs,
                entry_point: "main",
            });
            let depth_stencil_state = wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState {
                    ..Default::default()
                },
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
                depth_compare: wgpu::CompareFunction::Less,
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                stencil: wgpu::StencilState::default(),
            };

            Self::create_pipeline(
                fragment_state,
                &light_vs,
                &device,
                &layout,
                &[ModelVertex::desc()],
                Some("Light Render pipeline"),
                Some(depth_stencil_state),
            )
        };
        Self {
            device,
            surface,
            queue,
            sc_descriptor: sc_desc,
            swap_chain,
            size,
        }
    }
    fn create_pipeline(
        fragment_state: Option<wgpu::FragmentState>,
        vs_module: &wgpu::ShaderModule,
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
        label: Option<&str>,
        depth_stencil_state: Option<DepthStencilState>,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &vertex_buffer_layouts,
                module: &vs_module,
                entry_point: "main",
            },
            fragment: fragment_state,
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                strip_index_format: if cfg!(target_arch = "wasm32") {
                    Some(wgpu::IndexFormat::Uint32)
                } else {
                    None
                },
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            multisample: wgpu::MultisampleState {
                ..Default::default()
            },
            depth_stencil: depth_stencil_state,
        })
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_descriptor.height = new_size.height;
        self.sc_descriptor.width = new_size.width;
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.sc_descriptor, "depth_texture");
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.sc_descriptor);
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.y / (self.size.height as f64),
                    g: position.x / (self.size.width as f64),
                    b: 1.0,
                    a: 1.0,
                };

                true
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    if keycode == VirtualKeyCode::Space {
                        for handles in self.world.read_component::<PhysicsHandle>().join() {
                            let mut physicsworld = self.world.fetch_mut::<PhysicsWorld>();
                            physicsworld
                                .body_set
                                .get_mut(handles.rigid_body_handle)
                                .unwrap()
                                .apply_impulse(glm::vec3(0.0, 120.0, -5.0), true);
                        }
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
    pub fn update(&mut self) {
        let mut movement = movement::LightTransform;
        let mut physics = Physics;
        physics.run_now(&self.world);
        movement.run_now(&self.world);
        self.world.maintain();
        for (i, transform) in (
            &self.world.read_component::<LightHandle>(),
            &self.world.read_component::<Transform>(),
        )
            .join()
        {
            let light = self.lights.get_mut(i.index).unwrap();
            light.direction = transform.position;
            self.queue.write_buffer(
                &self.light_buffer,
                (i.index * std::mem::size_of::<DirectionalLightRaw>()) as wgpu::BufferAddress,
                bytemuck::bytes_of(&light.to_raw()),
            );
        }
        // move this to it's system also once the state has been refactored to be a resource for ECS
        let instance_data: Vec<TransformRaw> = (
            &self.world.read_component::<Transform>(),
            &self.world.read_component::<PhysicsHandle>(),
        )
            .join()
            .map(|(transform, _physics_handle)| Transform::to_raw(transform))
            .collect();
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
        let normal_matricies = instance_data
            .iter()
            .map(TransformRaw::get_normal_matrix)
            .collect::<Vec<_>>();
        self.queue
            .write_buffer(&self.normals, 0, bytemuck::cast_slice(&normal_matricies));
    }
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });
        encoder.push_debug_group("shadow passes");

        for (i, light) in self.lights.iter().enumerate() {
            // copy light's viewproj matrix to shadow uniform
            encoder.copy_buffer_to_buffer(
                &self.light_buffer,
                (i * std::mem::size_of::<DirectionalLightRaw>()) as wgpu::BufferAddress,
                &self.shadow_pass.uniform_buffer,
                0,
                64,
            );

            // render entities from each of the light's point of view
            encoder.insert_debug_marker("render entities");
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("shadow pass descriptor"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &light.target_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        },
                    ),
                });
                pass.set_pipeline(&self.shadow_pass.pipeline);
                pass.set_bind_group(0, &self.shadow_pass.bind_group, &[]);

                let mesh = &self.obj_model.meshes[0]; // we assume there is at least one mesh
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.element_count, 0, 0..self.instances.len() as u32);
            }
            encoder.pop_debug_group();
        }

        encoder.pop_debug_group();
        encoder.push_debug_group("forward render pass");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("forward pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.light_render_pipeline);

            render_pass.draw_light_model_instanced(
                &self.obj_model,
                0..self.lights.len() as u32,
                &self.uniform_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            let mesh = &self.obj_model.meshes[0]; // we assume there is at least one mesh
            let material = &self.obj_model.materials[mesh.material];
            render_pass.draw_mesh_instanced(
                mesh,
                0..self.instances.len() as u32,
                material,
                &self.uniform_bind_group,
                &self.light_bind_group,
            );
        }
        encoder.pop_debug_group();

        // {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
        //             attachment: &frame.view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.1,
        //                     g: 0.2,
        //                     b: 0.3,
        //                     a: 1.0,
        //                 }),
        //                 store: true,
        //             },
        //         }],
        //         depth_stencil_attachment: None,
        //         label: Some("texture renderer"),
        //     });
        //     render_pass.set_pipeline(&self.texture_renderer.render_pipeline);
        //     render_pass.set_bind_group(0, &self.texture_renderer.bind_group, &[]);
        //     render_pass.set_vertex_buffer(0, self.texture_renderer.quad.slice(..));
        //     render_pass.draw(0..TextureRenderer::QUAD_VERTEX_ARRAY.len() as u32, 0..1);
        // }

        self.queue.submit(std::iter::once(encoder.finish()));

        let now = chrono::offset::Utc::now();
        self.total_frame_time = self.total_frame_time.add(Duration::nanoseconds(
            (now - self.previous_frame_time).timestamp_nanos(),
        ));
        if self.total_frame_time < Duration::seconds(1) {
            self.frame_count += 1;
        } else {
            log::info!("FPS: {}", self.frame_count);
            self.frame_count = 0;
            self.total_frame_time = Duration::seconds(0);
        }
        self.previous_frame_time = Duration::nanoseconds(now.timestamp_nanos());

        Ok(())
    }
}
