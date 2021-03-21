pub mod shadowpass;

pub trait RenderPass<T> {
    fn create(device: &wgpu::Device) -> Self;
    fn execute(&mut self);
}
