use egui_wgpu_backend::RenderPass;

pub struct EguiContainer {
    pub render_pass: RenderPass,
    pub state: egui_winit::State,
    pub context: egui::Context,
}
impl EguiContainer {
    pub fn handle_events(&mut self, event: &winit::event::WindowEvent<'_>) -> bool {
        self.state.on_event(&self.context, event)
    }
}
