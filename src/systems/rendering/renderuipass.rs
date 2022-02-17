use egui_wgpu_backend::ScreenDescriptor;
use epi::{IntegrationInfo, WebInfo};
use specs::{ReadExpect, System, WriteExpect};

use crate::{DeltaTime, renderer::state::State, resources::eguirenderpass::EguiRenderPass};
use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::resources::eguicontainer::EguiContainer;

pub struct RenderUIPass;

impl<'a> System<'a> for RenderUIPass {
    type SystemData = (WriteExpect<'a, EguiContainer>,ReadExpect<'a,State>,WriteExpect<'a,HorizonCommandEncoder>);

    fn run(&mut self, (mut egui_container,state,mut command_encoder): Self::SystemData) {
        egui_container.platform.begin_frame();
        egui::Window::new("Window").show(&egui_container.platform.context(),|ui|{
            ui.label("Hello world");
        });
        let (output, paint_commands) = egui_container.platform.end_frame(None);
        let paint_jobs = egui_container.platform.context().tessellate(paint_commands);
        let encoder = command_encoder.get_encoder();
        let screen_desc = ScreenDescriptor {
            scale_factor:state.scale_factor as f32,
            physical_height: state.sc_descriptor.height,
            physical_width: state.sc_descriptor.width,
        };
        let font_image = egui_container.platform.context().font_image();
        let  render_pass = &mut egui_container.render_pass.pass;
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
