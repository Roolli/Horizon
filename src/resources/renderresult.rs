pub struct RenderResult {
    pub result: Option<wgpu::SurfaceError>,
}
impl Default for RenderResult {
    fn default() -> Self {
        Self { result: None }
    }
}
