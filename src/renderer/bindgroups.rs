use wgpu::Device;

use super::bindgroupcontainer::{self, BindGroupContainer};

pub mod lighting;
pub mod shadow;
pub mod uniforms;

pub trait HorizonBindGroup {
    fn get_layout(device: &Device) -> wgpu::BindGroupLayout;

    fn create_container(device: &Device) -> BindGroupContainer;
}
