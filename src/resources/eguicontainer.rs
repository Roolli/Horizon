use std::sync::Arc;
use egui_winit_platform::Platform;
use epi::App;
use epi::backend::RepaintSignal;
use crate::CustomEvent;
use crate::resources::eguirenderpass::EguiRenderPass;

pub struct EguiContainer {
    pub render_pass: EguiRenderPass,
    pub platform: Platform,
}
