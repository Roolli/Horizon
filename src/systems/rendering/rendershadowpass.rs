use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect};
use wgpu::BufferAddress;

use crate::{BufferTypes, components::transform::{Transform}, Instances, renderer::{
    bindgroupcontainer::BindGroupContainer,
    bindgroups::shadow::ShadowBindGroup,
    model::{HorizonModel},
    pipelines::{shadowpipeline::ShadowPipeline},
    state::State,
}, resources::{
    bindingresourcecontainer::BindingResourceContainer, commandencoder::HorizonCommandEncoder,
}, ShadowUniform};
use crate::components::transform::TransformRaw;
use crate::resources::bindingresourcecontainer::TextureViewTypes;

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
            .buffers[BufferTypes::DirectionalLight].as_ref().unwrap();
        let shadow_uniform_buf = binding_resource_container
            .buffers[ShadowUniform].as_ref().unwrap();
        cmd_encoder.copy_buffer_to_buffer(dir_light_buf, 0, shadow_uniform_buf, 0, 64);

        cmd_encoder.insert_debug_marker("render_entities");
        let mut pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow pass descriptor"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: binding_resource_container
                    .texture_views[TextureViewTypes::Shadow].as_ref().unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0f32),
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
        let mut begin_instance_index :u32 = 0;
        for (model, model_ent) in (&models, &*entities).join() {
            let mut instance_buffer = Vec::new();
            for transform in transforms.join() {
                if let Some(model) = transform.model {
                    if model_ent == model {
                        instance_buffer.push(transform.to_raw());
                    }
                }
            }
            state.queue.write_buffer(
                binding_resource_container
                    .buffers[Instances].as_ref().unwrap(),
                (std::mem::size_of::<TransformRaw>() * begin_instance_index as usize) as BufferAddress,
                bytemuck::cast_slice(&instance_buffer),
            );
            // TODO: FIX
            // for mesh in &model.meshes {
            //     pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            //     pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            //     pass.draw_indexed(0..mesh.element_count, 0, begin_instance_index..begin_instance_index + instance_buffer.len() as u32);
            // }
            begin_instance_index += instance_buffer.len() as u32;
        }
        drop(pass);
    }
}
