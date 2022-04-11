use crate::BufferTypes::{DebugCollisionUniform, DebugCollisionVertex};
use crate::{BindGroupContainer, BindingResourceContainer, HorizonBindGroup};
use specs::*;
use wgpu::{BindGroupLayout, Device};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct DebugCollisionBindGroup;

impl<'a> HorizonBindGroup<'a> for DebugCollisionBindGroup {
    type BindingResources = (&'a wgpu::Buffer);

    fn get_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("debug collision_debug_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: true,
                },
            }],
        })
    }

    fn create_container(device: &Device, resources: Self::BindingResources) -> BindGroupContainer {
        let layout = Self::get_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    offset: 0,
                    buffer: resources,
                    size: wgpu::BufferSize::new(
                        std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress
                    ),
                }),
            }],
            label: Some("debug collision bind group"),
            layout: &layout,
        });
        BindGroupContainer::new(layout, bind_group)
    }

    fn get_resources(device: &Device, resource_container: &mut BindingResourceContainer) {
        let debug_collision_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("debug_collision_vertex_buffer"),
            mapped_at_creation: false,
            size: (std::mem::size_of::<[f32; 3]>() * (u32::MAX / 512) as usize)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        let alignment = device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let size = std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress;
        let uniform_alignment = {
            let remainder = size % alignment;
            if remainder != 0 {
                size + alignment - remainder
            } else {
                size
            }
        };
        let uniform_buffer_size = uniform_alignment * 1024;
        let debug_collision_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("debug_collision_uniform_buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            size: uniform_buffer_size,
        });
        resource_container.buffers[DebugCollisionVertex] = Some(debug_collision_vertex_buffer);
        resource_container.buffers[DebugCollisionUniform] = Some(debug_collision_uniform_buffer);
    }
}
