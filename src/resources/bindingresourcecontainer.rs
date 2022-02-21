use std::collections::HashMap;
use enum_map::{EnumMap, Enum};

pub struct BindingResourceContainer {
    pub buffers: EnumMap<BufferTypes, Option<wgpu::Buffer>>,
    pub textures: EnumMap<TextureTypes, Option< wgpu::Texture>>,
    pub texture_views: EnumMap<TextureViewTypes, Option<wgpu::TextureView>>,
    pub samplers: EnumMap<SamplerTypes, Option<wgpu::Sampler>>,
}

impl Default for BindingResourceContainer {
    fn default() -> Self {
        Self {
            buffers: EnumMap::default(),
            textures: EnumMap::default(),
            texture_views: EnumMap::default(),
            samplers: EnumMap::default(),
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
}

#[derive(Enum)]
pub enum SamplerTypes
{
    Shadow,
    DeferredTexture,
    Skybox,
}

#[derive(Enum)]
pub enum TextureTypes
{
PositionDiffuseNormals,
Albedo,
Shadow,
Skybox,
}

#[derive(Enum)]
pub enum TextureViewTypes {
    Shadow,
    DeferredPosition,
    DeferredNormals,
    DeferredAlbedo,
    Skybox,
}
