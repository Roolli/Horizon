use super::HorizonBindGroup;
use crate::renderer::{bindgroups::BindGroupContainer, primitives::uniforms::ShadowUniforms};

use specs::*;
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct ShadowBindGroup;

impl<'a> HorizonBindGroup<'a> for ShadowBindGroup {
    type BindingResources = (&'a wgpu::Buffer, &'a wgpu::Buffer);
    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX,
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
        let shadow_bind_group_layout = Self::get_layout(&device);
        let (shadow_uniform_buffer, instance_buffer) = binding_resources;
        // ! move

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
            label: None,
        });

        BindGroupContainer::new(shadow_bind_group_layout, bind_group)
    }

    fn get_binding_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let shadow_uniforms_size = std::mem::size_of::<ShadowUniforms>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: shadow_uniforms_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
        });

        resource_container
            .buffers
            .insert(String::from("shadow_uniform_buffer"), uniform_buffer);
    }
}
