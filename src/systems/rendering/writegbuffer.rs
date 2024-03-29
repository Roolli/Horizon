use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect};
use wgpu::{BufferAddress, LoadOp, PipelineStatisticsTypes, QueryType};

use crate::components::gltfmodel::DrawModel;
use crate::resources::gpuquerysets::{
    GpuQuerySet, GpuQuerySetContainer, PipelineStatisticsQueries, TimestampQueries,
};
use crate::ui::gpustats::Passes;
use crate::TextureViewTypes::DeferredSpecular;
use crate::{
    components::transform::{Transform, TransformRaw},
    renderer::{
        bindgroupcontainer::BindGroupContainer, bindgroups::uniforms::UniformBindGroup,
        pipelines::gbufferpipeline::GBufferPipeline, state::State,
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
        Entities<'a>,
        WriteExpect<'a, GpuQuerySetContainer>,
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
            entities,
            mut query_sets,
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
        if let Some(ref query_set) = query_sets.container {
            render_pass
                .write_timestamp(&query_set.timestamp_queries, query_set.next_query_index * 2);
            render_pass.begin_pipeline_statistics_query(
                &query_set.pipeline_queries,
                query_set.next_query_index,
            );
        }

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
        if let Some(ref mut query_set) = query_sets.container {
            render_pass.write_timestamp(
                &query_set.timestamp_queries,
                query_set.next_query_index * 2 + 1,
            );
            render_pass.end_pipeline_statistics_query();
            query_set
                .pass_indices
                .insert(Passes::GBuffer, query_set.next_query_index);
            query_set.next_query_index += 1;
        }
        drop(render_pass);

        encoder.finish(&state.device, &state.queue);
    }
}
