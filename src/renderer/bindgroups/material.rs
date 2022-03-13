use wgpu::{BindGroupLayout, Device};
use crate::{BindGroupContainer, BindingResourceContainer, HorizonBindGroup, Texture};

pub struct MaterialBindGroup;

impl<'a> HorizonBindGroup<'a> for MaterialBindGroup {
    type BindingResources = (&'a Texture,&'a Texture,&'a Texture,&'a Texture,&'a Texture,&'a wgpu::Buffer);

    fn get_layout(device: &Device) -> BindGroupLayout {
        // Sampler might not be needed for every texture
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { // base color texture
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { //roughness texture
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{ // normal map texture
                    binding:3,
                    visibility:wgpu::ShaderStages::FRAGMENT,
                    ty:wgpu::BindingType::Texture {
                        multisampled:false,
                        sample_type: wgpu::TextureSampleType::Float {filterable:true},
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{ // occlusion map texture
                    binding:4,
                    visibility:wgpu::ShaderStages::FRAGMENT,
                    ty:wgpu::BindingType::Texture {
                        multisampled:false,
                        sample_type: wgpu::TextureSampleType::Float {filterable:true},
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{ // emissive map texture
                    binding:5,
                    visibility:wgpu::ShaderStages::FRAGMENT,
                    ty:wgpu::BindingType::Texture {
                        multisampled:false,
                        sample_type: wgpu::TextureSampleType::Float {filterable:true},
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:6,
                    visibility:wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset:false,
                    },
                    count: None,
                }
            ],
            label: Some("diffuse texture bind group layout"),
        })
    }

    fn create_container(device: &Device, (base_color_texture,roughness_texture,normal_map,occlusion_texture,emissive_texture,material_uniforms): Self::BindingResources) -> BindGroupContainer {
        let bind_group_layout = Self::get_layout(device);
      let bind_group =  device.create_bind_group(&wgpu::BindGroupDescriptor {
         layout: &bind_group_layout,
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&base_color_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&base_color_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&roughness_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&occlusion_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&emissive_texture.view),
                },
                wgpu::BindGroupEntry{
                    binding:6,
                    resource: wgpu::BindingResource::Buffer(material_uniforms.as_entire_buffer_binding())
                }
            ],
        });
            BindGroupContainer::new(bind_group_layout,bind_group)
    }

    fn get_resources(device: &Device, resource_container: &mut BindingResourceContainer) {
    }
}