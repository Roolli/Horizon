use super::HorizonBindGroup;
use crate::renderer::{
    bindgroups::BindGroupContainer, primitives::uniforms::Globals, state::State,
};

use specs::*;
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct UniformBindGroup;

impl<'a> HorizonBindGroup<'a> for UniformBindGroup {
    type BindingResources = (
        &'a wgpu::Sampler,
        &'a wgpu::TextureView,
        &'a wgpu::Buffer,
        &'a wgpu::Buffer,
        &'a wgpu::Buffer,
    );
    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Sampler {
                        comparison: true,
                        filtering: false,
                    },
                },
            ],
        })
    }

    fn create_container(
        device: &wgpu::Device,
        binding_resources: Self::BindingResources,
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let (sampler, texture_view, uniform_buffer, normal_buffer, instance_buffer) =
            binding_resources;

        let uniform_bind_group_layout = UniformBindGroup::get_layout(&device);
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: normal_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            layout: &uniform_bind_group_layout,
        });

        BindGroupContainer::new(uniform_bind_group_layout, uniform_bind_group)
    }

    fn get_binding_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: State::SHADOW_SIZE,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::RENDER_ATTACHMENT,
            label: Some("shadow texture"),
            mip_level_count: 1,
            sample_count: 1,
        });

        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        let normal_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            mapped_at_creation: false,
            label: Some("model_matrix_buffer"),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::STORAGE,
            size: State::MAX_ENTITY_COUNT,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance_buffer"),
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
            size: State::MAX_ENTITY_COUNT,
        });

        let uniform_size = std::mem::size_of::<Globals>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: Some("uniform_buffer"),
            size: uniform_size,
            mapped_at_creation: false,
        });
        resource_container
            .samplers
            .insert(String::from("shadow_sampler"), shadow_sampler);
        resource_container
            .textures
            .insert(String::from("shadow_texture"), shadow_texture);
        resource_container
            .texture_views
            .insert(String::from("shadow_view"), shadow_view);
        resource_container
            .buffers
            .insert(String::from("normal_buffer"), normal_matrix_buffer);
        resource_container
            .buffers
            .insert(String::from("instance_buffer"), instance_buffer);
        resource_container
            .buffers
            .insert(String::from("uniform_buffer"), uniform_buffer);
    }
}
