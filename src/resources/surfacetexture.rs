#[derive(Default)]
pub struct SurfaceTexture {
    pub texture: Option<wgpu::SurfaceTexture>,
}