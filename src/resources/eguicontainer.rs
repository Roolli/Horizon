use egui_wgpu_backend::RenderPass;
use egui_winit_platform::Platform;

pub struct EguiContainer {
    pub render_pass: RenderPass,
    pub platform: Platform,
}
