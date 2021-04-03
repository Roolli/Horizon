use crate::renderer::utils::texturerenderer::TextureRenderer;
use crate::{
    components::transform::{Transform, TransformRaw},
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::{lighting::LightBindGroup, uniforms::UniformBindGroup},
        model::{DrawModel, HorizonModel},
        pipelines::{forwardpipeline::ForwardPipeline, RenderPipelineBuilder},
        primitives::{lights::directionallight::DirectionalLight, uniforms::Globals},
        state::State,
    },
    resources::{
        bindingresourcecontainer::BindingResourceContainer, camera::Camera,
        commandencoder::HorizonCommandEncoder, windowevents::ResizeEvent,
    },
};
use futures::io::ReadExact;
use image::flat::View;
use specs::prelude::*;
use wgpu::{CommandEncoder, CommandEncoderDescriptor, SwapChainError};
pub struct RenderForwardPass;

impl<'a> System<'a> for RenderForwardPass {
    type SystemData = (
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, State>,
        ReadExpect<'a, BindingResourceContainer>,
        ReadStorage<'a, LightBindGroup>,
        ReadStorage<'a, UniformBindGroup>,
        ReadStorage<'a, BindGroupContainer>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, HorizonModel>,
        Entities<'a>,
        ReadExpect<'a, ForwardPipeline>,
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
            transforms,
            models,
            entities,
            forward_pipeline,
        ): Self::SystemData,
    ) {
        let frame = state.swap_chain.get_current_frame().unwrap().output;
        let cmd_encoder = encoder.get_encoder();

        let mut render_pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("forward pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &state.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });

        // let renderer = TextureRenderer::new(
        //     &state.device,
        //     binding_resource_container
        //         .texture_views
        //         .get("shadow_view")
        //         .unwrap(),
        //     &state.sc_descriptor,
        // );
        // let mut render_pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     color_attachments: &[wgpu::RenderPassColorAttachment {
        //         view: &frame.view,
        //         resolve_target: None,
        //         ops: wgpu::Operations {
        //             load: wgpu::LoadOp::Clear(wgpu::Color {
        //                 r: 0.1,
        //                 g: 0.2,
        //                 b: 0.3,
        //                 a: 1.0,
        //             }),
        //             store: true,
        //         },
        //     }],
        //     depth_stencil_attachment: None,
        //     label: Some("texture renderer"),
        // });

        // render_pass.set_pipeline(&renderer.render_pipeline);
        // render_pass.set_bind_group(0, &renderer.bind_group, &[]);
        // render_pass.set_vertex_buffer(0, renderer.quad.slice(..));
        // render_pass.draw(0..TextureRenderer::QUAD_VERTEX_ARRAY.len() as u32, 0..1);

        render_pass.set_pipeline(&forward_pipeline.0);

        //TODO: Convert to resource
        let (_, uniform_bind_group_container) = (&uniform_bind_group, &bind_group_containers)
            .join()
            .next()
            .unwrap();
        let (_, light_bind_group_container) = (&light_bind_group, &bind_group_containers)
            .join()
            .next()
            .unwrap();
        // TODO: move to it's own system

        // // // TODO: move this to it's own system
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
            //     // TODO: change to resource
            render_pass.draw_model_instanced(
                &model,
                0..instance_buffer.len() as u32,
                &uniform_bind_group_container.bind_group,
                &light_bind_group_container.bind_group,
            );
        }
        drop(render_pass);
        encoder.finish(&state.device, &state.queue);

        // encoder.finish(&state.device, &state.queue);
        // let finished_encoder = *encoder;
        // state.queue.submit(std::iter::once(encoder.finish()));
        // *encoder = HorizonCommandEncoder::new(state.device.create_command_encoder(
        //     &CommandEncoderDescriptor {
        //         label: Some("encoder"),
        //     },
        // ));
    }
}
