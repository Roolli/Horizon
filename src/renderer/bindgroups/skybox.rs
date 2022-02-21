
use specs::*;
use wgpu::{BindGroupLayout, BufferUsages, Device, ShaderStages};
use crate::{BindingResourceContainer, HorizonBindGroup};
use crate::renderer::bindgroupcontainer::BindGroupContainer;

use crate::renderer::primitives::uniforms::SkyboxUniform;

#[derive(Default,Component)]
#[storage(NullStorage)]
pub struct SkyboxBindGroup;
impl SkyboxBindGroup{
    pub const IMAGE_SIZE: u32 = 128;
}
impl<'a> HorizonBindGroup<'a> for SkyboxBindGroup {
    type BindingResources = (&'a wgpu::Buffer,&'a wgpu::TextureView,&'a wgpu::Sampler);

    fn get_layout(device: &Device) -> BindGroupLayout {
         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label:Some("Skybox Bind group"),
             entries:&[
                 wgpu::BindGroupLayoutEntry{
                     binding:0,
                     visibility:  ShaderStages::VERTEX_FRAGMENT,
                     count:None,
                     ty: wgpu::BindingType::Buffer {
                         ty: wgpu::BufferBindingType::Uniform,
                         has_dynamic_offset:false,
                         min_binding_size:None,
                     }
                 },
                 wgpu::BindGroupLayoutEntry {
                     binding:1,
                     visibility: ShaderStages::FRAGMENT,
                     count:None,
                     ty:wgpu::BindingType::Texture {
                         multisampled:false,
                         view_dimension: wgpu::TextureViewDimension::Cube,
                         sample_type: wgpu::TextureSampleType::Float {filterable:true}
                     }
                 },
                 wgpu::BindGroupLayoutEntry {
                     binding:2,
                     visibility:wgpu::ShaderStages::FRAGMENT,
                     count:None,
                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
                 }
             ]
         })
    }

    fn create_container(device: &Device, resources: Self::BindingResources) -> BindGroupContainer {
            let (uniform,texture,sampler) = resources;
        let bind_group_layout = Self::get_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Skybox BindGroup"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding:0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry{
                    binding:1,
                    resource: wgpu::BindingResource::TextureView(&texture),
                },
                wgpu::BindGroupEntry{
                    binding:2,
                    resource: wgpu::BindingResource::Sampler(sampler)
                }
            ],
        });
        BindGroupContainer::new(bind_group_layout,bind_group)
    }

    fn get_resources(device: &Device, resource_container: &mut BindingResourceContainer) {
        let skybox_sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            label:Some("skybox_sampler"),
            address_mode_u:wgpu::AddressMode::ClampToEdge,
            address_mode_v:wgpu::AddressMode::ClampToEdge,
            address_mode_w:wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let size = wgpu::Extent3d{
            width: Self::IMAGE_SIZE,
            height: Self::IMAGE_SIZE,
            depth_or_array_layers: 6,
        };
        let layers= wgpu::Extent3d {
            depth_or_array_layers:1,
            ..size
        };
        let max_mips = layers.max_mips();
        // add texture at a later time / allow scripting to modify it.
       let skybox_texture =  device.create_texture(&wgpu::TextureDescriptor{
            label: Some("Skybox_Texture"),
            size,
           sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format:wgpu::TextureFormat::Bgra8UnormSrgb,
            mip_level_count: max_mips,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |wgpu::TextureUsages::COPY_DST,
        });
        let skybox_texture_view = skybox_texture.create_view(&wgpu::TextureViewDescriptor{
            label: Some("Skybox_Texture_View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..wgpu::TextureViewDescriptor::default()
        });
        let skybox_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Skybox Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM |BufferUsages::COPY_DST,
            size: std::mem::size_of::<SkyboxUniform>() as wgpu::BufferAddress,
            mapped_at_creation:false,
        });
        resource_container.textures.insert(stringify!(skybox_texture).to_string(),skybox_texture);
        resource_container.texture_views.insert(stringify!(skybox_texture_view).to_string(),skybox_texture_view);
        resource_container.buffers.insert(stringify!(skybox_buffer).to_string(),skybox_buffer);
        resource_container.samplers.insert(stringify!(skybox_sampler).to_string(),skybox_sampler);
    }
}
