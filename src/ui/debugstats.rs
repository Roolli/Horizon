use egui::{Context, CtxRef, Ui};
use crate::ui::{UiComponent, ViewComponent};

pub struct DebugStats
{
   pub fps:u16,
    pub unique_model_count: u32,
    pub messages: Vec<String>,
    pub debug_texture: Option<wgpu::Texture>,
    pub debug_texture_view: Option<wgpu::TextureView>
}

impl UiComponent for DebugStats
{
    fn name(&self) -> &'static str {
        "debug_stats"
    }

    fn show(&mut self, ctx: &CtxRef, open: &mut bool) {
        egui::Window::new(self.name()).collapsible(false).show(ctx,|ui|{
            use super::ViewComponent as _;
            self.ui(ui);
        });
    }
}

impl ViewComponent for DebugStats{
    fn ui(&mut self, ui: &mut Ui) {
        ui.separator();

        ui.horizontal(|ui|{
                ui.label(format!("FPS: {}",self.fps));
            for message in &self.messages {
                ui.label(message);
            }
        });

    }
}