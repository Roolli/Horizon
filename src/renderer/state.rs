use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::bindgroups::lighting::LightBindGroup;
use crate::renderer::bindgroups::shadow::ShadowBindGroup;
use crate::renderer::bindgroups::uniforms::UniformBindGroup;
use crate::renderer::bindgroups::HorizonBindGroup;
use crate::renderer::primitives::lights::directionallight::DirectionalLight;
use crate::renderer::primitives::lights::directionallight::DirectionalLightRaw;
use crate::renderer::primitives::lights::pointlight::PointLightRaw;
use crate::renderer::primitives::lights::spotlight::SpotLightRaw;

use crate::systems::movement;
use crate::{
    components::transform,
    renderer::primitives::{texture::Texture, vertex::Vertex},
    systems::physics::{Physics, PhysicsWorld},
};
use crate::{filesystem::modelimporter::Importer, renderer::model::HorizonModel};
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

use chrono::{Duration, DurationRound, Timelike};

use nalgebra::Isometry3;
use rapier3d::{
    dynamics::RigidBodyBuilder,
    geometry::{ColliderBuilder, TriMesh},
};
use specs::{Builder, Join, RunNow, World, WorldExt};

use winit::{event::*, window::Window};

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_descriptor: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: Texture,
}
impl State {
    //TODO: Move this to a constants / limits struct for cleanliness
    pub const OPENGL_TO_WGPU_MATRIX: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ];
    pub const MAX_ENTITY_COUNT: wgpu::BufferAddress =
        (std::mem::size_of::<TransformRaw>() * ((u16::MAX / 2) as usize)) as wgpu::BufferAddress;
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

        Self {
            depth_texture: Texture::create_depth_texture(&device, &sc_desc, "depth_texture"),
            device,
            surface,
            queue,
            sc_descriptor: sc_desc,
            swap_chain,
            size,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {}
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => true,
            WindowEvent::KeyboardInput { input, .. } => {
                // if let Some(keycode) = input.virtual_keycode {
                //     if keycode == VirtualKeyCode::Space {
                //         for handles in self.world.read_component::<PhysicsHandle>().join() {
                //             let mut physicsworld = self.world.fetch_mut::<PhysicsWorld>();
                //             physicsworld
                //                 .body_set
                //                 .get_mut(handles.rigid_body_handle)
                //                 .unwrap()
                //                 .apply_impulse(glm::vec3(0.0, 120.0, -5.0), true);
                //         }
                //         return true;
                //     }
                // }

                false
            }
            _ => false,
        }
    }
    pub fn update(&mut self) {}
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        Ok(())
        // encoder.push_debug_group("shadow passes");

        // for (i, light) in self.lights.iter().enumerate() {
        //     // copy light's viewproj matrix to shadow uniform
        //     encoder.copy_buffer_to_buffer(
        //         &self.light_buffer,
        //         (i * std::mem::size_of::<DirectionalLightRaw>()) as wgpu::BufferAddress,
        //         &self.shadow_pass.uniform_buffer,
        //         0,
        //         64,
        //     );

        //     // render entities from each of the light's point of view
        //     encoder.insert_debug_marker("render entities");
        //     {
        //         let mut pass = encoder.begin_render_pass();
        //         pass.set_pipeline(&self.shadow_pass.pipeline);
        //         pass.set_bind_group(0, &self.shadow_pass.bind_group, &[]);

        //         let mesh = &self.obj_model.meshes[0]; // we assume there is at least one mesh
        //         pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        //         pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //         pass.draw_indexed(0..mesh.element_count, 0, 0..self.instances.len() as u32);
        //     }
        //     encoder.pop_debug_group();
        // }

        // encoder.pop_debug_group();
        // encoder.push_debug_group("forward render pass");
        // {
        //     render_pass.set_pipeline(&self.light_render_pipeline);

        //     render_pass.draw_light_model_instanced(
        //         &self.obj_model,
        //         0..self.lights.len() as u32,
        //         &self.uniform_bind_group,
        //         &self.light_bind_group,
        //     );

        //     render_pass.set_pipeline(&self.render_pipeline);
        //     let mesh = &self.obj_model.meshes[0]; // we assume there is at least one mesh
        //     let material = &self.obj_model.materials[mesh.material];
        //     render_pass.draw_mesh_instanced(
        //         mesh,
        //         0..self.instances.len() as u32,
        //         material,
        //         &self.uniform_bind_group,
        //         &self.light_bind_group,
        //     );
        // }
        // encoder.pop_debug_group();

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

        //

        // let now = chrono::offset::Utc::now();
        // self.total_frame_time = self.total_frame_time.add(Duration::nanoseconds(
        //     (now - self.previous_frame_time).timestamp_nanos(),
        // ));
        // if self.total_frame_time < Duration::seconds(1) {
        //     self.frame_count += 1;
        // } else {
        //     log::info!("FPS: {}", self.frame_count);
        //     self.frame_count = 0;
        //     self.total_frame_time = Duration::seconds(0);
        // }
        // self.previous_frame_time = Duration::nanoseconds(now.timestamp_nanos());

        // Ok(())
    }
}
