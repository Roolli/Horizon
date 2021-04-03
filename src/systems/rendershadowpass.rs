use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect};

use crate::{
    components::transform::{Transform, TransformRaw},
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::shadow::ShadowBindGroup,
        model::{DrawModel, HorizonModel},
        pipelines::{shadowpipeline::ShadowPipeline, RenderPipelineBuilder},
        state::State,
    },
    resources::{
        bindingresourcecontainer::BindingResourceContainer, commandencoder::HorizonCommandEncoder,
    },
};
pub struct RenderShadowPass;
impl<'a> System<'a> for RenderShadowPass {
    type SystemData = (
        ReadExpect<'a, State>,
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, BindingResourceContainer>,
        Entities<'a>,
        ReadStorage<'a, HorizonModel>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, BindGroupContainer>,
        ReadStorage<'a, ShadowBindGroup>,
        ReadExpect<'a, ShadowPipeline>,
    );

    fn run(
        &mut self,
        (
            state,
            mut encoder,
            binding_resource_container,
            entities,
            models,
            transforms,
            bind_group_container,
            shadow_bind_group,
            shadow_pipeline,
        ): Self::SystemData,
    ) {
        let cmd_encoder = encoder.get_encoder();
        cmd_encoder.push_debug_group("shadow pass");
        // copy the light's view matrix to the shadow uniform buffer
        let dir_light_buf = binding_resource_container
            .buffers
            .get("directional_light_buffer")
            .unwrap();
        let shadow_uniform_buf = binding_resource_container
            .buffers
            .get("shadow_uniform_buffer")
            .unwrap();
        cmd_encoder.copy_buffer_to_buffer(dir_light_buf, 0, shadow_uniform_buf, 0, 64);
        cmd_encoder.insert_debug_marker("render_entities");
        let mut pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow pass descriptor"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: binding_resource_container
                    .texture_views
                    .get("shadow_view")
                    .unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        pass.set_pipeline(&shadow_pipeline.0);
        let (_, sh_pass_bind_group) = (&shadow_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();
        pass.set_bind_group(0, &sh_pass_bind_group.bind_group, &[]);
        for (model, model_ent) in (&models, &*entities).join() {
            let mut instance_buffer = Vec::new();
            for transform in transforms.join() {
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
            for mesh in &model.meshes {
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.element_count, 0, 0..instance_buffer.len() as u32);
            }
        }
        drop(pass);
    }
}
