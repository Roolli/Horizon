use super::texture::Texture;
use rapier3d::na::Vector4;

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub normal_texture: Texture,
}

#[derive(Clone, Copy)]
pub struct GltfMaterial {
    pub base_color: Vector4<f64>,
    pub base_color_texture: wgpu::Texture,
    pub pbr_roughness: f32,
    pub metallic_factor: f32,
    pub roughness_texture: wgpu::Texture,
    pub normal_map_texture: wgpu::Texture,
    pub double_sided: bool,
    pub occlusion_texture: wgpu::Texture,
    pub emissive_color: Vector4<f64>,
    pub emissive_texture: wgpu::Texture,
    pub unlit: bool,
}
