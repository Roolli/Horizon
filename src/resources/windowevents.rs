pub struct ResizeEvent {
    pub new_size: winit::dpi::PhysicalSize<u32>,
    pub handled: bool,
}
