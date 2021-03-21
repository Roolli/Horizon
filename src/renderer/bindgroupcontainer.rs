use specs::*;
use std::collections::HashMap;

use wgpu::{BindGroup, BindGroupLayout, Buffer};

/// Holds all relevant data that is associated with the given bind group.
#[derive(Component)]
#[storage(VecStorage)]
pub struct BindGroupContainer {
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl BindGroupContainer {
    pub fn new(layout: BindGroupLayout, bind_group: BindGroup) -> Self {
        Self { layout, bind_group }
    }
}
