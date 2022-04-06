use crate::{BindGroupContainer, BindingResourceContainer, HorizonBindGroup, SamplerTypes};
use specs::*;
use wgpu::{BindGroupLayout, Device};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct DebugTextureBindGroup;

impl<'a> HorizonBindGroup<'a> for DebugTextureBindGroup {
    type BindingResources = (&'a wgpu::TextureView, &'a wgpu::Sampler);

    fn get_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Renderer bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        })
    }

    fn create_container(
        device: &Device,
        (texture_view, sampler): Self::BindingResources,
    ) -> BindGroupContainer {
        let layout = Self::get_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("Texture_renderer bind_group"),
            layout: &layout,
        });
        BindGroupContainer::new(layout, bind_group)
    }

    fn get_resources(device: &Device, resource_container: &mut BindingResourceContainer) {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            ..Default::default()
        });
        resource_container.samplers[SamplerTypes::DebugTexture] = Some(sampler);
    }
}
