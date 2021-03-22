use specs::prelude::*;

use crate::{
    components::transform::{Transform, TransformRaw},
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::{lighting::LightBindGroup, uniforms::UniformBindGroup},
        model::{DrawModel, HorizonModel},
        pipelines::{forwardpipeline::ForwardPipeline, RenderPipelineContainer},
        state::State,
    },
    resources::{
        bindingresourcecontainer::BindingResourceContainer, commandencoder::HorizonCommandEncoder,
    },
};
pub struct RenderForwardPass;

impl<'a> System<'a> for RenderForwardPass {
    type SystemData = (
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, State>,
        ReadExpect<'a, BindingResourceContainer>,
        ReadStorage<'a, LightBindGroup>,
        ReadStorage<'a, UniformBindGroup>,
        ReadStorage<'a, BindGroupContainer>,
        ReadStorage<'a, ForwardPipeline>,
        ReadStorage<'a, RenderPipelineContainer>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, HorizonModel>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut encoder,
            state,
            binding_resource_container,
            light_bind_group,
            uniform_bind_group,
            bind_group_containers,
            forward_pipeline,
            render_pipeline_container,
            transforms,
            models,
            entities,
        ): Self::SystemData,
    ) {
        let frame = &state.swap_chain.get_current_frame().unwrap().output;
        let mut render_pass = encoder
            .cmd_encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("forward pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });
        let (_, forward_pipeline_container) = (&forward_pipeline, &render_pipeline_container)
            .join()
            .next()
            .unwrap();

        render_pass.set_pipeline(&forward_pipeline_container.pipeline);
        // TODO: move this to it's own system
        for (model, model_ent) in (&models, &*entities).join() {
            let mut instance_buffer = Vec::new();
            for (entity, transform) in (&*entities, &transforms).join() {
                if model_ent == transform.model {
                    instance_buffer.push(transform.to_raw());
                }
            }
            state.queue.write_buffer(
                binding_resource_container
                    .buffers
                    .get("instance_buffer")
                    .unwrap(),
                0,
                bytemuck::cast_slice(&instance_buffer),
            );
            let normal_matricies = instance_buffer
                .iter()
                .map(TransformRaw::get_normal_matrix)
                .collect::<Vec<_>>();
            state.queue.write_buffer(
                binding_resource_container
                    .buffers
                    .get("normal_buffer")
                    .unwrap(),
                0,
                bytemuck::cast_slice(&normal_matricies),
            );

            let (_, uniform_bind_group_container) = (&uniform_bind_group, &bind_group_containers)
                .join()
                .next()
                .unwrap();
            let (_, light_bind_group_container) = (&light_bind_group, &bind_group_containers)
                .join()
                .next()
                .unwrap();

            for mesh in &model.meshes {
                render_pass.draw_mesh_instanced(
                    mesh,
                    0..instance_buffer.len() as u32,
                    &model.materials[mesh.material],
                    &uniform_bind_group_container.bind_group,
                    &light_bind_group_container.bind_group,
                )
            }
        }
        drop(render_pass);
    }
}
