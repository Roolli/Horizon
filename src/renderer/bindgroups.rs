use wgpu::Device;

use crate::resources::bindingresourcecontainer::BindingResourceContainer;

use super::bindgroupcontainer::{self, BindGroupContainer};

pub mod lighting;
pub mod shadow;
pub mod uniforms;

pub trait HorizonBindGroup<'a> {
    type BindingResources;
    fn get_layout(device: &Device) -> wgpu::BindGroupLayout;

    fn create_container(device: &Device, resources: Self::BindingResources) -> BindGroupContainer;
    /// Gets (most) of the required binding resources associated with this given bind group
    /// NOTE: there may be cases where another bind group requires a binding resource, that's why
    /// there's a global container and you are required to get all binding resources for every available bind group to prevent issues.
    fn get_binding_resources(device: &Device, resource_container: &mut BindingResourceContainer);
}
