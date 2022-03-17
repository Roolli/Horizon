
use enum_map::*;
use serde::de::Unexpected::Enum;
use crate::Texture;


pub struct DefaultTextureContainer {
    pub elements: EnumMap<DefaultTextureTypes,Texture>,
}

impl DefaultTextureContainer {
    pub fn create(device:&wgpu::Device,queue:&wgpu::Queue) ->Self{

        let map = enum_map! {
            DefaultTextureTypes::MetallicRoughness => {
                Texture::create_default_texture_with_color(device,queue,[255,255,255],Some("default-metallic-roughness-texture"),false).unwrap()
            },
            DefaultTextureTypes::Emissive => {
                Texture::create_default_texture_with_color(device,queue,[255,255,255],Some("default-emissive-texture"),false).unwrap()
            },
            DefaultTextureTypes::Occlusion=> {
                Texture::create_default_texture_with_color(device,queue,[255,255,255],Some("default-occlusion-texture"),false).unwrap()
            },
            DefaultTextureTypes::NormalMap => {
                Texture::create_default_texture_with_color(device,queue,[0,0,255],Some("default-normal-texture"),true).unwrap()
            },
            DefaultTextureTypes::BaseColor=> {
            Texture::create_default_texture_with_color(device,queue,[255,255,255],Some("default-diffuse-texture"),false).unwrap()
            },
        };
        Self {
            elements: map
        }
    }
}

#[derive(Enum)]
pub enum DefaultTextureTypes {
    BaseColor,
    MetallicRoughness,
    NormalMap,
    Occlusion,
    Emissive,
}

