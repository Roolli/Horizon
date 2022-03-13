use image::DynamicImage;
use super::texture::Texture;
use bytemuck::*;

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub normal_texture: Texture,
}
//TODO: add clearcoat, & transmission
#[derive(Clone,Debug)]
pub struct GltfMaterial {
    pub base_color: [f32;4],
    pub base_color_texture: Option<DynamicImage>,
    pub pbr_roughness: f32,
    pub metallic_factor: f32,
    pub roughness_texture: Option<DynamicImage>,
    pub normal_map_texture: Option<DynamicImage>,
    pub double_sided: bool,
    pub occlusion_texture: Option<DynamicImage>,
    pub emissive_color: [f32;3],
    pub emissive_texture: Option<DynamicImage>,
    pub unlit: bool,
    pub name:String,
}
impl GltfMaterial {
    pub fn to_raw_material(&self) ->MaterialUniform
    {
        let double_sided = if  self.double_sided
        {
            -1.0
        }
        else {
            1.0
        };
        MaterialUniform {
            emissive_color: [self.emissive_color[0],self.emissive_color[1],self.emissive_color[2],1.0],
            roughness_metallic_double_sided: [self.pbr_roughness,self.metallic_factor,double_sided,0.0],
            base_color_factor: self.base_color,
        }
    }
}

#[derive(Pod,Zeroable,Copy,Clone)]
#[repr(C)]
pub struct MaterialUniform
{
    pub base_color_factor: [f32;4],
    pub roughness_metallic_double_sided:[f32;4],
    pub emissive_color:[f32;4],
}
