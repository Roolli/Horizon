use specs::Component;
use specs::NullStorage;

use crate::BufferTypes::{LightCulling, LightId};
use wgpu::util::DeviceExt;
use wgpu::BufferUsages;

use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::primitives::uniforms::{LightCullingUniforms, TileInfo};
use crate::Tiling;

use super::HorizonBindGroup;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct TilingBindGroup;

impl<'a> HorizonBindGroup<'a> for TilingBindGroup {
    type BindingResources = (
        &'a wgpu::Buffer,
        &'a wgpu::Buffer,
        &'a wgpu::Buffer,
        &'a wgpu::Buffer,
    );

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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
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
        let (
            tiling_uniform_buffer,
            canvas_constants_buffer,
            light_culling_uniforms,
            light_id_buffer,
        ) = resources;
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_culling_uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_id_buffer.as_entire_binding(),
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
        let light_culling_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light culling uniform buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<LightCullingUniforms>() as wgpu::BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let mut tile_info = TileInfo::default();
        let light_id_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light_id_buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: tile_info.calculate_light_id_buffer_size(600.0, 800.0),
        });

        let tiling_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tiling buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::bytes_of(&tile_info),
        });
        resource_container.buffers[LightId] = Some(light_id_buffer);
        resource_container.buffers[LightCulling] = Some(light_culling_buffer);
        resource_container.buffers[Tiling] = Some(tiling_buffer);
    }
}
