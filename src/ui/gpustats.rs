// use crate::ui::{UiComponent, ViewComponent};
// use egui::{Context, Ui};
// use std::collections::HashMap;
//
// pub struct GpuStats {
//     pub pipeline_data: HashMap<Passes, String>,
// }
//
#[derive(Eq, PartialEq, Hash)]
pub enum Passes {
    ShadowPassWithCascade(u32),
    GBuffer,
    LightCulling,
    Forward,
    Collision,
    Skybox,
    Ui,
}
// impl ViewComponent for GpuStats {
//     fn ui(&mut self, ui: &mut Ui) {
//
//     }
// }
// impl UiComponent for GpuStats {
//     fn name(&self) -> &'static str {
//         "GPU Pipeline Statistics"
//     }
//
//     fn show(&mut self, ctx: &Context, open: &mut bool) {
//         egui::Window::new(self.name()).show(ctx, |ui| {
//             self.ui(ui);
//         });
//     }
// }
