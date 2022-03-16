use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect};
use wgpu::BufferAddress;

use crate::{BufferTypes, Camera, components::transform::{Transform}, DirectionalLight, Instances, Projection, RawModel, renderer::{
    bindgroupcontainer::BindGroupContainer,
    bindgroups::shadow::ShadowBindGroup,
    model::{HorizonModel},
    pipelines::{shadowpipeline::ShadowPipeline},
    state::State,
}, resources::{
    bindingresourcecontainer::BindingResourceContainer, commandencoder::HorizonCommandEncoder,
}, ShadowUniform};

use crate::components::transform::TransformRaw;
use crate::renderer::primitives::uniforms::ShadowUniforms;
use crate::resources::bindingresourcecontainer::*;


pub struct RenderShadowPass;
impl<'a> System<'a> for RenderShadowPass {
    type SystemData = (
        ReadExpect<'a, State>,
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, BindingResourceContainer>,
        Entities<'a>,
        ReadStorage<'a, RawModel>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, BindGroupContainer>,
        ReadStorage<'a, ShadowBindGroup>,
        ReadExpect<'a, ShadowPipeline>,
        ReadExpect<'a, DirectionalLight>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a,Projection>
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
            dir_light,
            camera,
            projection,
        ): Self::SystemData,
    ) {
        let cmd_encoder = encoder.get_encoder();
        // get a new frustum for every cascade texture
        let shadow_uniform_buf = binding_resource_container
            .buffers[ShadowUniform].as_ref().unwrap();
        let raw_dir_lights = dir_light.get_view_and_proj_matrices(&camera,State::CASCADE_DISTS.0,State::CASCADE_DISTS.1,&projection);

        let shadow_cascade_buffer = binding_resource_container.buffers[BufferTypes::ShadowCascade].as_ref().unwrap();
        let cascade_lengths_buffer = binding_resource_container.buffers[BufferTypes::ShadowCascadeLengths].as_ref().unwrap();
        state.queue.write_buffer(cascade_lengths_buffer,0,bytemuck::cast_slice(raw_dir_lights.iter().map(|v|v.0).collect::<Vec<_>>().as_slice()));
        state.queue.write_buffer(shadow_cascade_buffer,0,bytemuck::cast_slice(raw_dir_lights.iter().map(|v| v.1.data.0).collect::<Vec<_>>().as_slice()));

        for (index,cascade) in binding_resource_container.texture_array_views[TextureArrayViewTypes::Shadow].iter().enumerate() {
            let format = format!("shadow pass for cascade: #{}",index);
            cmd_encoder.copy_buffer_to_buffer(shadow_cascade_buffer, (index * std::mem::size_of::<ShadowUniforms>()) as wgpu::BufferAddress, shadow_uniform_buf, 0, 64);
            let mut pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(format.as_str()),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: cascade,
                    depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0f32),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
            pass.insert_debug_marker("render_entities");

        pass.set_pipeline(&shadow_pipeline.0);
        let (_, sh_pass_bind_group) = (&shadow_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();
        pass.set_bind_group(0, &sh_pass_bind_group.bind_group, &[]);
        let mut begin_instance_index: u32 = 0;
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
            for mesh in &model.meshes {
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_buffer_len, 0, begin_instance_index..begin_instance_index + instance_buffer.len() as u32);
            }

            begin_instance_index += instance_buffer.len() as u32;
        }
     }
    }
}
