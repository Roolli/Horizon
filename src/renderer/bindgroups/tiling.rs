
use specs::Component;
use specs::NullStorage;

use wgpu::BufferUsages;

use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::primitives::uniforms::TileInfo;
use crate::Tiling;

use super::HorizonBindGroup;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct TilingBindGroup;

impl<'a> HorizonBindGroup<'a> for TilingBindGroup {
    type BindingResources = (&'a wgpu::Buffer, &'a wgpu::Buffer);

    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                },
            ],
            label: Some("Tiling bind group layout"),
        })
    }

    fn create_container(
        device: &wgpu::Device,
        resources: Self::BindingResources,
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let (tiling_uniform_buffer, canvas_constants_buffer) = resources;
        let tiling_group_layout = Self::get_layout(device);
        let tiling_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: tiling_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: canvas_constants_buffer.as_entire_binding(),
                },
            ],
            label: Some("Tiling bind group"),
            layout: &tiling_group_layout,
        });
        BindGroupContainer::new(tiling_group_layout, tiling_bind_group)
    }

    fn get_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let tiling_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tiling buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<TileInfo>() as wgpu::BufferAddress,
            usage: BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        resource_container
            .buffers[Tiling] =Some(tiling_buffer);
    }
}
