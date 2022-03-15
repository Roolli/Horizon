use std::num::NonZeroU32;
use super::HorizonBindGroup;
use crate::renderer::{bindgroups::BindGroupContainer, primitives::uniforms::ShadowUniforms};

use specs::*;
use crate::{SamplerTypes, ShadowUniform, State, TextureTypes, TextureViewTypes};
use crate::resources::bindingresourcecontainer::TextureArrayViewTypes;
use std::default::Default;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct ShadowBindGroup;

impl<'a> HorizonBindGroup<'a> for ShadowBindGroup {
    type BindingResources = (&'a wgpu::Buffer, &'a wgpu::Buffer);
    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            ],
        })
    }
    fn create_container(
        device: &wgpu::Device,
        binding_resources: Self::BindingResources,
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let shadow_bind_group_layout = Self::get_layout(device);
        let (shadow_uniform_buffer, instance_buffer) = binding_resources;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
            ],
            label: Some("shadow_bind_group"),
        });

        BindGroupContainer::new(shadow_bind_group_layout, bind_group)
    }

    fn get_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let shadow_uniforms_size = std::mem::size_of::<ShadowUniforms>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: shadow_uniforms_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::GreaterEqual),
            ..Default::default()
        });

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: State::SHADOW_SIZE,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("shadow texture"),
            mip_level_count: 1,
            sample_count: 1,
        });
        (0..State::SHADOW_SIZE.depth_or_array_layers).for_each(|i|{
            let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                dimension:Some(wgpu::TextureViewDimension::D2),
                base_mip_level:0,
                base_array_layer: i,
                aspect:wgpu::TextureAspect::All,
                mip_level_count:None,
                label:Some(format!("Shadow cascade {}",i).as_str()),
                array_layer_count: NonZeroU32::new(1),
                format:None,
            });
            resource_container
                .texture_array_views[TextureArrayViewTypes::Shadow].push(shadow_view);
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        resource_container
            .buffers[ShadowUniform]=Some(uniform_buffer);


        resource_container
            .samplers[SamplerTypes::Shadow] = Some(shadow_sampler);
        resource_container
            .textures[TextureTypes::Shadow] = Some(shadow_texture);
        resource_container.texture_views[TextureViewTypes::Shadow] = Some(shadow_view);

    }
}
