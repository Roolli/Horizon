use std::collections::HashMap;

use wgpu::{BindGroup, BindGroupLayout, Buffer};

/// Holds all relevant data that is associated with the given bind group.

pub struct BindGroupContainer {
    /// the buffers the bind_group requires
    pub buffers: HashMap<String, Buffer>,
    pub textures: HashMap<String, wgpu::Texture>,
    pub samplers: HashMap<String, wgpu::Sampler>,
    pub texture_views:HashMap<String, wgpu::TextureView>
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl BindGroupContainer {
    pub fn new(layout: BindGroupLayout, bind_group: BindGroup) -> Self {
        Self {
            buffers: HashMap::new(),
            samplers: HashMap::new(),
            textures: HashMap::new(),
            texture_views: HashMap::new(),
            layout,
            bind_group,
        }
    }
    pub fn add_buffer(&mut self, label: String, buffer: wgpu::Buffer) {
        self.buffers.insert(label, buffer);
    }
    pub fn add_sampler(&mut self, label: String, sampler: wgpu::Sampler) {
        self.samplers.insert(label, sampler);
    }
    pub fn add_texture(&mut self, label: String, texture: wgpu::Texture) {
        self.textures.insert(label, texture);
    }
    pub fn add_texture_view(&mut self, label: String, texture: wgpu::TextureView) {
        self.texture_views.insert(label, texture);
    }
}
