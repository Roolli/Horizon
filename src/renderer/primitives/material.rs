use super::texture::Texture;

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub normal_texture: Texture,
}
