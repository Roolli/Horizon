use std::collections::HashMap;
use enum_map::{EnumMap, Enum};

pub struct BindingResourceContainer {
    pub buffers: EnumMap<BufferTypes, Option<wgpu::Buffer>>,
    pub textures: EnumMap<TextureTypes, Option< wgpu::Texture>>,
    pub texture_views: EnumMap<TextureViewTypes, Option<wgpu::TextureView>>,
    pub samplers: EnumMap<SamplerTypes, Option<wgpu::Sampler>>,
    pub texture_array_views:EnumMap<TextureArrayViewTypes,Vec<wgpu::TextureView>>,
}

impl Default for BindingResourceContainer {
    fn default() -> Self {
        Self {
            buffers: EnumMap::default(),
            textures: EnumMap::default(),
            texture_views: EnumMap::default(),
            samplers: EnumMap::default(),
            texture_array_views:EnumMap::default(),
        }
    }
}

#[derive(Enum)]
pub enum BufferTypes {
    CanvasSize,
    DeferredVao,
    Uniform,
    Normals,
    Instances,
    ShadowUniform,
    DirectionalLight,
    PointLight,
    SpotLight,
    Tiling,
    Skybox,
    DebugTextureVertex
}

#[derive(Enum)]
pub enum SamplerTypes
{
    Shadow,
    DeferredTexture,
    Skybox,
    DebugTexture,
}

#[derive(Enum)]
pub enum TextureTypes
{
PositionDiffuseNormals,
Albedo,
Shadow,
Skybox,
}

#[derive(Enum,Debug,PartialEq,Copy,Clone)]
pub enum TextureViewTypes {
    DeferredPosition,
    DeferredNormals,
    DeferredAlbedo,
    DeferredSpecular,
    Skybox,
    Shadow,
    Depth,
}
#[derive(Enum,Debug,PartialOrd, PartialEq,Copy,Clone)]
pub enum TextureArrayViewTypes{
    Shadow
}