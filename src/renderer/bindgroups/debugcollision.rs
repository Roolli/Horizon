use wgpu::{BindGroupLayout, Device};
use crate::{BindGroupContainer, BindingResourceContainer, HorizonBindGroup};

pub struct DebugCollisonBindGroup;

impl HorizonBindGroup<'a> for DebugCollisonBindGroup {
    type BindingResources = ();

    fn get_layout(device: &Device) -> BindGroupLayout {
    }

    fn create_container(device: &Device, resources: Self::BindingResources) -> BindGroupContainer {
        todo!()
    }

    fn get_resources(device: &Device, resource_container: &mut BindingResourceContainer) {
        todo!()
    }
}