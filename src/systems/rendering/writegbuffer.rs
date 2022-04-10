use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect};
use wgpu::{BufferAddress, LoadOp};

use crate::components::gltfmodel::DrawModel;
use crate::TextureViewTypes::DeferredSpecular;
use crate::{
    components::transform::{Transform, TransformRaw},
    renderer::{
        bindgroupcontainer::BindGroupContainer, bindgroups::uniforms::UniformBindGroup,
        model::HorizonModel, pipelines::gbufferpipeline::GBufferPipeline, state::State,
    },
    resources::{
        bindingresourcecontainer::BindingResourceContainer, commandencoder::HorizonCommandEncoder,
    },
    DeferredAlbedo, DeferredNormals, DeferredPosition, EguiContainer, Instances, Normals, RawModel,
};

pub struct WriteGBuffer;

impl<'a> System<'a> for WriteGBuffer {
    type SystemData = (
        ReadExpect<'a, BindingResourceContainer>,
        ReadExpect<'a, State>,
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadStorage<'a, UniformBindGroup>,
        ReadStorage<'a, BindGroupContainer>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, RawModel>,
        ReadExpect<'a, GBufferPipeline>,
        WriteExpect<'a, EguiContainer>,
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
                    view: binding_resource_container.texture_views[DeferredPosition]
                        .as_ref()
                        .unwrap(),
                },
                wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0f64,
                            g: 0f64,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                    view: binding_resource_container.texture_views[DeferredNormals]
                        .as_ref()
                        .unwrap(),
                },
                wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.0f64,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                    view: binding_resource_container.texture_views[DeferredSpecular]
                        .as_ref()
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
                    view: binding_resource_container.texture_views[DeferredAlbedo]
                        .as_ref()
                        .unwrap(),
                },
            ],
        });
        let (_, uniform_bind_group_container) = (&uniform_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();
        render_pass.set_bind_group(0, &uniform_bind_group_container.bind_group, &[]);
        render_pass.set_pipeline(&gbuffer_pipeline.0);
        let mut begin_instance_index: u32 = 0;

        let query_set = state.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Timestamp QuerySet"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });

        let timestamp_period = state.queue.get_timestamp_period();

        let query_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("query_buffer"),
            size: (std::mem::size_of::<[u64; 2]>() as wgpu::BufferAddress)
                .max(wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        render_pass.write_timestamp(&query_set, 0);

        for (model, model_ent) in (&models, &*entities).join() {
            let mut instance_buffer: Vec<TransformRaw> = Vec::new();
            for transform in transforms.join() {
                if let Some(model) = transform.model {
                    if model_ent == model {
                        instance_buffer.push(transform.to_raw());
                    }
                }
            }

            state.queue.write_buffer(
                binding_resource_container.buffers[Instances]
                    .as_ref()
                    .unwrap(),
                (std::mem::size_of::<TransformRaw>() * begin_instance_index as usize)
                    as BufferAddress,
                bytemuck::cast_slice(&instance_buffer),
            );

            let normal_matrices = instance_buffer
                .iter()
                .map(TransformRaw::get_normal_matrix)
                .collect::<Vec<_>>();
            state.queue.write_buffer(
                binding_resource_container.buffers[Normals]
                    .as_ref()
                    .unwrap(),
                (std::mem::size_of::<TransformRaw>() * begin_instance_index as usize)
                    as BufferAddress,
                bytemuck::cast_slice(&normal_matrices),
            );

            render_pass.draw_model_instanced(
                model,
                begin_instance_index..begin_instance_index + instance_buffer.len() as u32,
            );
            begin_instance_index += instance_buffer.len() as u32;
        }
        render_pass.write_timestamp(&query_set, 1);
        drop(render_pass);
        #[cfg(not(target_arch = "wasm32"))]
        cmd_encoder.resolve_query_set(&query_set, 0..2, &query_buffer, 0);
        encoder.finish(&state.device, &state.queue);
        if !cfg!(target_arch = "wasm32") {
            let _ = query_buffer.slice(..).map_async(wgpu::MapMode::Read);
            state.device.poll(wgpu::Maintain::Wait);
            let timestamp_view = query_buffer
                .slice(..std::mem::size_of::<[u64; 2]>() as wgpu::BufferAddress)
                .get_mapped_range();
            let timestamp_data: &[u64; 2] = bytemuck::from_bytes(&*timestamp_view);

            let nanos = (timestamp_data[1] - timestamp_data[0]) as f32 * timestamp_period;
            let micros = nanos / 1000.0;
            log::info!(target:"performance","gbuffer generation took {:.3} μs", micros);
        }
    }
}
