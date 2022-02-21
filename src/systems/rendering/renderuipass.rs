use egui_wgpu_backend::ScreenDescriptor;
use epi::{IntegrationInfo, WebInfo};
use specs::{ReadExpect, System, WriteExpect};
use wgpu::{FilterMode, Texture, TextureView};

use crate::{BindingResourceContainer, DeltaTime, renderer::state::State};
use crate::renderer::utils::texturerenderer::TextureRenderer;
use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::eguicontainer::EguiContainer;
use crate::ui::debugstats::DebugStats;
use crate::ui::UiComponent;

pub struct RenderUIPass;

impl<'a> System<'a> for RenderUIPass {
    type SystemData = (WriteExpect<'a, EguiContainer>,ReadExpect<'a,State>,WriteExpect<'a,HorizonCommandEncoder>,WriteExpect<'a,DebugStats>,ReadExpect<'a,BindingResourceContainer>);

    fn run(&mut self, (mut egui_container,state,mut command_encoder,mut debug_ui,binding_resource_container): Self::SystemData) {

        debug_ui.show(&egui_container.platform.context(),&mut true);
        if debug_ui.debug_texture.is_none()
        {
            let albedo_texture:Option<Texture> = Some(state.device.create_texture(&wgpu::TextureDescriptor {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                mip_level_count: 1,
                label: Some("albedo_texture"),
                sample_count: 1,
                size: wgpu::Extent3d {
                    depth_or_array_layers: 1,
                    height: state.sc_descriptor.height,
                    width: state.sc_descriptor.width,
                },
            }));
            debug_ui.debug_texture =albedo_texture;
        }
       if debug_ui.debug_texture_view.is_none()
       {
           let albedo_view:Option<TextureView> = Some(debug_ui.debug_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor{
               array_layer_count: std::num::NonZeroU32::new(1),
               base_array_layer: 0,
               ..Default::default()
           }));
           debug_ui.debug_texture_view = albedo_view;
       }

        let encoder = command_encoder.get_encoder();
        {
            let renderer = TextureRenderer::new_depth_texture_visualizer(
                &state.device,
                    &state.depth_texture.view,
                &state.sc_descriptor,
            );
            // binding_resource_container
            //     .texture_views
            //     .get("shadow_view")
            //     .unwrap(),

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &debug_ui.debug_texture_view.as_ref().unwrap(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
                label: Some("texture renderer"),
            });

            render_pass.set_pipeline(&renderer.1);
            render_pass.set_bind_group(0, &renderer.0, &[]);
            render_pass.draw(0..6, 0..1);
        }

        let image_id = egui_container.render_pass.egui_texture_from_wgpu_texture(&state.device,&debug_ui.debug_texture.as_ref().unwrap(),FilterMode::Linear);
        egui::Window::new("depth_view").show(&egui_container.platform.context(),|ui|{
            ui.image(image_id,egui::Vec2::new(640.0,480.0));
        });
        let (output, paint_commands) = egui_container.platform.end_frame(None);
        let paint_jobs = egui_container.platform.context().tessellate(paint_commands);

        let screen_desc = ScreenDescriptor {
            scale_factor:state.scale_factor as f32,
            physical_height: state.sc_descriptor.height,
            physical_width: state.sc_descriptor.width,
        };
        let font_image = egui_container.platform.context().font_image();
        let  render_pass = &mut egui_container.render_pass;
        render_pass.update_texture(&state.device,&state.queue,&font_image);
        render_pass.update_user_textures(&state.device,&state.queue);
        render_pass.update_buffers(&state.device,&state.queue,&paint_jobs,&screen_desc);
        let output_frame = state.surface.get_current_texture().unwrap();
        let color_attachment = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        //TODO: add error resource and return the error if anything happens.
        render_pass.execute(encoder,&color_attachment,&paint_jobs,&screen_desc,None).unwrap();
        command_encoder.finish(&state.device,&state.queue);
        output_frame.present();

    }
}
