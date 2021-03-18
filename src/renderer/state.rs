use std::{
    num::NonZeroU32,
    ops::{Add, Deref},
};

use crate::{
    components::transform,
    renderer::primitives::{texture::Texture, vertex::Vertex},
    systems::physics::{Physics, PhysicsWorld},
};
use crate::{filesystem::modelimporter::Importer, renderer::model::HorizonModel};
use crate::{renderer::cam::Camera, systems::movement};

use super::{
    light::{DrawLight, Light, LightHandle, LightRaw},
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
use wgpu::{util::DeviceExt, BufferBindingType, DepthStencilState, ShaderFlags, ShaderStage};
use winit::{event::*, window::Window};

// used to measure if we're in 1 second or not

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    pub queue: wgpu::Queue,
    sc_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    camera: Camera,
    instances: Vec<Transform>,
    instance_buffer: wgpu::Buffer,
    normals: wgpu::Buffer,
    depth_texture: Texture,
    obj_model: HorizonModel,
    lights: Vec<Light>,
    light_bind_group: wgpu::BindGroup,
    light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_buffer: wgpu::Buffer,
    light_render_pipeline: wgpu::RenderPipeline,
    uniforms: Globals,
    pub world: specs::World,
    frame_count: u32,
    previous_frame_time: chrono::Duration,
    total_frame_time: chrono::Duration,
    shadow_pass: Pass,
    texture_renderer: TextureRenderer,
}

const NUM_INSTANCES_PER_ROW: u32 = 15;

impl State {
    pub const OPENGL_TO_WGPU_MATRIX: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ];
    const MAX_ENTITY_COUNT:wgpu::BufferAddress = (u16::MAX / 4) as wgpu::BufferAddress;
    const MAX_LIGHTS: usize = 10;
    const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        depth_or_array_layers: Self::MAX_LIGHTS as u32,
        height: 4096,
        width: 4096,
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

        //! Method should END HERE

        // SHADOW
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: Self::SHADOW_SIZE,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::RENDER_ATTACHMENT,
            label: Some("shadow texture"),
            mip_level_count: 1,
            sample_count: 1,
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut texture_views = (0..2)
            .map(|i| {
                Some(shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("shadow"),
                    array_layer_count: NonZeroU32::new(1),
                    format: None,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i as u32,
                    level_count: None,
                }))
            })
            .collect::<Vec<_>>();

        // Lights
        //TODO: Change to directional light, add spot / point lights
        let lights = vec![
            Light::new(
                glm::vec3(-100.0, 50.0, 100.0),
                wgpu::Color {
                    r: 1.0,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                },
                90.0,
                0.1..1000.0,
                texture_views[0].take().unwrap(),
            ),
            // Light::new(
            //     glm::vec3(10.0, 5.0, 10.0),
            //     wgpu::Color {
            //         r: 0.3,
            //         g: 0.6,
            //         b: 0.2,
            //         a: 0.0,
            //     },
            //     45.0,
            //     0.1..50.0,
            //     texture_views[1].take().unwrap(),
            // ),
        ];

        let normal_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            mapped_at_creation: false,
            label: Some("Model matricies"),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::STORAGE,
            size:  Self::MAX_ENTITY_COUNT,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance buffer"),
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation:false,
            size: Self::MAX_ENTITY_COUNT,
        });

        let uniform_size = std::mem::size_of::<Globals>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: Some("Uniform buffer"),
           size: uniform_size,
           mapped_at_creation:false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::VERTEX,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Sampler {
                            comparison: true,
                            filtering: false,
                        },
                    },
                ],
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: normal_matrix_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
            layout: &uniform_bind_group_layout,
        });

        let light_uniform_size =
            (Self::MAX_LIGHTS * std::mem::size_of::<LightRaw>()) as wgpu::BufferAddress;
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Vertex Buffer"),
            mapped_at_creation: false,
            size: light_uniform_size,
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            label: None,
            entries: &[wgpu::BindGroupEntry {
                resource: light_buffer.as_entire_binding(),
                binding: 0,
            }],
        });

        let shadow_pass = {
            let shadow_uniforms_size = std::mem::size_of::<ShadowUniforms>() as wgpu::BufferAddress;
            let shadow_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::VERTEX,
                            count: None,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(shadow_uniforms_size),
                            },
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::VERTEX,
                            count: None,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                        },
                    ],
                });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow"),
                bind_group_layouts: &[&shadow_bind_group_layout],
                push_constant_ranges: &[],
            });

            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: shadow_uniforms_size,
                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &shadow_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: instance_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            });
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
        let texture_renderer = TextureRenderer::new(&device, &lights[0].target_view, &sc_desc);
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

        world.register::<LightHandle>();
        for (i, light) in lights.iter().enumerate() {
            world
                .create_entity()
                .with(Transform {
                    position: light.pos,
                    rotation: glm::quat_angle_axis(0.0, &glm::vec3(0.0, 0.0, 1.0)),
                    scale: glm::vec3(0.25, 0.25, 0.25),
                })
                .with(LightHandle { index: i })
                .build();
        }

        Self {
            normals: normal_matrix_buffer,
            depth_texture: Texture::create_depth_texture(&device, &sc_desc, "depth_texture"),
            instances,
            camera: cam,
            device,
            surface,
            queue,
            sc_descriptor: sc_desc,
            swap_chain,
            size,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            render_pipeline: basic_pipeline,
            instance_buffer,
            uniform_bind_group,
            uniform_buffer,
            obj_model,
            lights,
            light_bind_group,
            light_bind_group_layout,
            light_buffer,
            light_render_pipeline,
            uniforms: globals,
            world,
            shadow_pass,
            frame_count: 0,
            previous_frame_time: Duration::nanoseconds(
                chrono::offset::Utc::now().timestamp_nanos(),
            ),
            total_frame_time: Duration::seconds(0),
            texture_renderer,
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
            light.pos = transform.position;
            self.queue.write_buffer(
                &self.light_buffer,
                (i.index * std::mem::size_of::<LightRaw>()) as wgpu::BufferAddress,
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
                (i * std::mem::size_of::<LightRaw>()) as wgpu::BufferAddress,
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
