use std::collections::HashMap;
pub struct BindingResourceContainer {
    pub buffers: HashMap<String, wgpu::Buffer>,
    pub textures: HashMap<String, wgpu::Texture>,
    pub texture_views: HashMap<String, wgpu::TextureView>,
    pub samplers: HashMap<String, wgpu::Sampler>,
}

impl Default for BindingResourceContainer {
    fn default() -> Self {
        Self {
            buffers: HashMap::new(),
            textures: HashMap::new(),
            texture_views: HashMap::new(),
            samplers: HashMap::new(),
        }
    }
}
