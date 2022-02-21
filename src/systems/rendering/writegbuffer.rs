use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect};
use wgpu::{BufferAddress, LoadOp};

use crate::{components::transform::{Transform, TransformRaw}, EguiContainer, renderer::{
    bindgroupcontainer::BindGroupContainer,
    bindgroups::uniforms::UniformBindGroup,
    model::{HorizonModel},
    pipelines::gbufferpipeline::GBufferPipeline,
    state::State,
}, resources::{
    bindingresourcecontainer::BindingResourceContainer, commandencoder::HorizonCommandEncoder,
}};

pub struct WriteGBuffer;

impl<'a> System<'a> for WriteGBuffer {
    type SystemData = (
        ReadExpect<'a, BindingResourceContainer>,
        ReadExpect<'a, State>,
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadStorage<'a, UniformBindGroup>,
        ReadStorage<'a, BindGroupContainer>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, HorizonModel>,
        ReadExpect<'a, GBufferPipeline>,
        WriteExpect<'a,EguiContainer>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            binding_resource_container,
            state,
            mut encoder,
            uniform_bind_group,
            bind_group_container,
            transforms,
            models,
            gbuffer_pipeline,
           mut egui_container,
            entities,
        ): Self::SystemData,
    ) {
        //TODO: move egui begin to separate system... maybe.
        egui_container.platform.begin_frame();

        let cmd_encoder = encoder.get_encoder();

        let mut render_pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("writeGBuffer"),
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0f32),
                    store: true,
                }),
                stencil_ops: None,
                view: &state.depth_texture.view,
            }),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: f64::MAX,
                            g: f64::MAX,
                            b: f64::MAX,
                            a: 1.0,
                        }),
                        store: true,
                    },
                    view: binding_resource_container
                        .texture_views
                        .get("position_view")
                        .unwrap(),
                },
                wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0f64,
                            g: 0f64,
                            b: 0f64,
                            a: 1.0,
                        }),
                        store: true,
                    },
                    view: binding_resource_container
                        .texture_views
                        .get("normal_view")
                        .unwrap(),
                },
                wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0f64,
                            g: 0f64,
                            b: 0f64,
                            a: 1.0,
                        }),
                        store: true,
                    },
                    view: binding_resource_container
                        .texture_views
                        .get("albedo_view")
                        .unwrap(),
                },
            ],
        });
        let mut begin_instance_index:u32 = 0;
        for (model, model_ent) in (&models, &*entities).join() {
            let mut instance_buffer:Vec<TransformRaw> = Vec::new();
            for transform in transforms.join() {
                if let Some(model) = transform.model {
                    if model_ent == model {
                        instance_buffer.push(transform.to_raw());
                    }
                }
            }

            state.queue.write_buffer(
                binding_resource_container
                    .buffers
                    .get("instance_buffer")
                    .unwrap(),
                (std::mem::size_of::<TransformRaw>()  *begin_instance_index as usize) as BufferAddress,
                bytemuck::cast_slice(&instance_buffer),
            );

            let normal_matrices = instance_buffer
                .iter()
                .map(TransformRaw::get_normal_matrix)
                .collect::<Vec<_>>();
            state.queue.write_buffer(
                binding_resource_container
                    .buffers
                    .get("normal_buffer")
                    .unwrap(),
                (std::mem::size_of::<TransformRaw>()  *begin_instance_index as usize) as BufferAddress,
                bytemuck::cast_slice(&normal_matrices),
            );
            let (_, uniform_bind_group_container) = (&uniform_bind_group, &bind_group_container)
                .join()
                .next()
                .unwrap();
            render_pass.set_pipeline(&gbuffer_pipeline.0);

            render_pass.set_bind_group(1, &uniform_bind_group_container.bind_group, &[]);
            for mesh in &model.meshes {
                render_pass.set_bind_group(0, &model.materials[mesh.material].bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.element_count, 0, begin_instance_index..begin_instance_index + instance_buffer.len() as u32);
            }
            begin_instance_index += instance_buffer.len() as u32;

        }
        drop(render_pass);
        encoder.finish(&state.device, &state.queue);
    }
}
