use std::collections::HashMap;
use image::DynamicImage;
use super::texture::Texture;
use bytemuck::*;
use enum_map::EnumMap;
use gltf::material::AlphaMode;
use wgpu::util::DeviceExt;
use crate::{BindGroupContainer, DefaultTextureTypes, HorizonBindGroup, MaterialBindGroup};


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
    pub base_color_texture: Option<usize>,
    pub pbr_roughness: f32,
    pub metallic_factor: f32,
    pub roughness_texture: Option<usize>,
    pub normal_map_texture: Option<usize>,
    pub double_sided: bool,
    pub occlusion_texture: Option<usize>,
    pub emissive_color: [f32;3],
    pub emissive_texture: Option<usize>,
    pub unlit: bool,
    pub alpha_mode: AlphaMode,
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
    pub fn register_bind_group(&self,device:&wgpu::Device,loaded_textures:&HashMap<usize,Texture>,default_textures:&EnumMap<DefaultTextureTypes,Texture>) -> BindGroupContainer
    {
        let diffuse_texture =  if let Some(base_tex_id) = self.base_color_texture
        {
            loaded_textures.get(&base_tex_id).unwrap()
        }
        else {
            &default_textures[DefaultTextureTypes::BaseColor]
        };
        let normal_map =  if let Some(normal_id) = self.normal_map_texture
        {
            loaded_textures.get(&normal_id).unwrap()
        }
        else {
            &default_textures[DefaultTextureTypes::NormalMap]
        };
        let occlusion_texture =  if let Some(occulison) = self.occlusion_texture
        {
            loaded_textures.get(&occulison).unwrap()
        }
        else {
            &default_textures[DefaultTextureTypes::Occlusion]
        };
        let roughness_texture =  if let Some(roughness) = self.roughness_texture
        {
            loaded_textures.get(&roughness).unwrap()
        }
        else {
            &default_textures[DefaultTextureTypes::MetallicRoughness]
        };
        let emissive_texture =  if let Some(emissive) = self.emissive_texture
        {
            loaded_textures.get(&emissive).unwrap()
        }
        else {
            &default_textures[DefaultTextureTypes::Emissive]
        };
        let material_uniforms = self.to_raw_material();
        let material_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label:Some(format!("material-uniform-{}",self.name).as_str()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::bytes_of(&material_uniforms)
        });
        MaterialBindGroup::create_container(&device, (&diffuse_texture,&roughness_texture, &normal_map,&occlusion_texture, &emissive_texture,&material_uniform))
    }
    pub fn upload_material_textures_to_gpu(&self,device:&wgpu::Device,queue:&wgpu::Queue,image_list:&HashMap<usize,DynamicImage>,gpu_texture_container: &mut HashMap<usize,Texture>)
    {
        if let Some(base_color_tex) = self.base_color_texture
        {
            if !gpu_texture_container.contains_key(&base_color_tex)
            {
                gpu_texture_container.insert(base_color_tex,Self::load_texture_from_image(format!("diffuse-{}",self.name).as_str(),&device,&queue,&image_list[&base_color_tex],false));
            }
        }
        if let Some(normal_tex) = self.normal_map_texture
        {
            if !gpu_texture_container.contains_key(&normal_tex)
            {
                gpu_texture_container.insert(normal_tex, Self::load_texture_from_image(format!("normal-{}", self.name).as_str(), &device, &queue, &image_list[&normal_tex], true));
            }
        }
        if let Some(metallic_roughness) = self.roughness_texture
        {
            if !gpu_texture_container.contains_key(&metallic_roughness)
            {
                gpu_texture_container.insert(metallic_roughness, Self::load_texture_from_image(format!("metallic-roughness-{}", self.name).as_str(), &device, &queue, &image_list[&metallic_roughness], false));
            }
        }

        if let Some(emissive) = self.emissive_texture
        {
            if !gpu_texture_container.contains_key(&emissive)
            {
                gpu_texture_container.insert(emissive, Self::load_texture_from_image(format!("emissive-{}", self.name).as_str(), &device, &queue, &image_list[&emissive], false));
            }
        }
        if let Some(occlusion) = self.occlusion_texture
        {
            if !gpu_texture_container.contains_key(&occlusion)
            {
                gpu_texture_container.insert(occlusion, Self::load_texture_from_image(format!("occlusion-{}", self.name).as_str(), &device, &queue, &image_list[&occlusion], false));
            }
        }
    }

    fn load_texture_from_image(name: &str, device:&wgpu::Device,queue:&wgpu::Queue,image:&DynamicImage,is_normal:bool) -> Texture
    {
        Texture::from_image(device,queue,image,Some(name.to_string().as_str()),is_normal).unwrap()
        // else {
        //     //Texture::create_default_texture_with_color(device,queue,def_color,Some(format!("default-{}",name).as_str()),is_normal).unwrap()
        // }
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
