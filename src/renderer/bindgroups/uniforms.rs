use super::HorizonBindGroup;
use crate::renderer::{
    bindgroups::BindGroupContainer, primitives::uniforms::Globals, state::State,
};

use crate::resources::bindingresourcecontainer::{TextureTypes, TextureViewTypes};
use crate::{Instances, Normals, Shadow, Uniform};
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
        &'a wgpu::Buffer,
        &'a wgpu::Buffer,
    );
    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT
                        | wgpu::ShaderStages::COMPUTE,
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                // Currently web doesn't allow for single texture view to be sent as a 2DArray
                if !cfg!(target_arch = "wasm32") {
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                    }
                } else {
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                    }
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison), // might not work if filtering not enabled
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
        let (
            sampler,
            texture_view,
            uniform_buffer,
            normal_buffer,
            instance_buffer,
            shadow_cascade_buffer,
            cascade_lengths,
        ) = binding_resources;

        let uniform_bind_group_layout = UniformBindGroup::get_layout(device);
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
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: shadow_cascade_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: cascade_lengths.as_entire_binding(),
                },
            ],
            layout: &uniform_bind_group_layout,
        });

        BindGroupContainer::new(uniform_bind_group_layout, uniform_bind_group)
    }

    fn get_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let normal_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            mapped_at_creation: false,
            label: Some("model_matrix_buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            size: State::MAX_ENTITY_COUNT,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: State::MAX_ENTITY_COUNT,
        });

        let uniform_size = std::mem::size_of::<Globals>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label: Some("uniform_buffer"),
            size: uniform_size,
            mapped_at_creation: false,
        });

        resource_container.buffers[Normals] = Some(normal_matrix_buffer);
        resource_container.buffers[Instances] = Some(instance_buffer);
        resource_container.buffers[Uniform] = Some(uniform_buffer);
    }
}
