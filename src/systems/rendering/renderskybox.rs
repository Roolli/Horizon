use specs::{Join, ReadExpect, ReadStorage, System, Write, WriteExpect};
use wgpu::{RenderPassColorAttachment, RenderPassDepthStencilAttachment};
use crate::{LightBindGroup, RenderResult, SkyboxBindGroup, SkyboxPipeline, State};
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::resources::commandencoder::HorizonCommandEncoder;

pub struct RenderSkyBox;

impl<'a> System<'a> for RenderSkyBox{
    type SystemData = (Write<'a,RenderResult>,ReadStorage<'a,BindGroupContainer>,ReadStorage<'a,SkyboxBindGroup>,ReadExpect<'a,SkyboxPipeline>,ReadExpect<'a,State>,WriteExpect<'a,HorizonCommandEncoder>);

    fn run(&mut self, (mut render_result,bind_group_container,skybox_bind_group,pipeline,state,mut command_encoder): Self::SystemData) {
        let frame_result = state.surface.get_current_texture();
        let frame;
        if let Ok(f) = frame_result {
            frame = f;
        } else {
            render_result.result = frame_result.err();
            return;
        }
          let frame_view = &frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let cmd_encoder = command_encoder.get_encoder();

        let mut render_pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: Some("SkyboxRenderPass"),
            color_attachments: &[
               RenderPassColorAttachment{
                   resolve_target: None,
                   ops: wgpu::Operations{
                       load: wgpu::LoadOp::Load,
                       store: true,
                   },
                   view: frame_view
               }
            ],
                depth_stencil_attachment:    Some(RenderPassDepthStencilAttachment {
                    view: &state.depth_texture.view,
                    depth_ops: None,
                    stencil_ops: None,
                }),
            });



        let (_, skybox_bind_group_container) = (&skybox_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();
        render_pass.set_bind_group(0,&skybox_bind_group_container.bind_group,&[]);
        render_pass.set_pipeline(&pipeline.0);
        render_pass.draw(0..3,0..1);
    }
}