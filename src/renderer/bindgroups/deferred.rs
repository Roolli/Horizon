use super::HorizonBindGroup;
use crate::renderer::{
    bindgroupcontainer::BindGroupContainer, primitives::uniforms::CanvasConstants,
};
use specs::*;
use wgpu::util::DeviceExt;
use crate::resources::bindingresourcecontainer::BufferTypes::{CanvasSize, DeferredVao};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct DeferredBindGroup;

impl DeferredBindGroup {
    const ARRAY: [[f32; 2]; 6] = [
        [-1.0, -1.0],
        [1.0, -1.0],
        [-1.0, 1.0],
        [-1.0, 1.0],
        [1.0, -1.0],
        [1.0, 1.0],
    ];
}

impl<'a> HorizonBindGroup<'a> for DeferredBindGroup {
    type BindingResources = (
        &'a wgpu::Sampler,
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        &'a wgpu::Buffer,
    );

    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("deferred bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                },
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 2,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 3,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 4,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding: 5,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_container(
        device: &wgpu::Device,
        resources: Self::BindingResources,
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let (sampler, position_texture,
            normals_texture, albedo_texture,specular_view, canvas_size_buffer) =
            resources;
        let deferred_bind_group_layout = Self::get_layout(device);
        let deferred_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &deferred_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(position_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(normals_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(specular_view),
                },
                wgpu::BindGroupEntry{
                    binding:4,
                    resource: wgpu::BindingResource::TextureView(albedo_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: canvas_size_buffer.as_entire_binding(),
                },
            ],
            label: Some("deferred_bind_group"),
        });

        BindGroupContainer::new(deferred_bind_group_layout, deferred_bind_group)
    }

    fn get_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let canvas_size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas_size_buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<CanvasConstants>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        resource_container.buffers[CanvasSize] =Some(canvas_size_buffer);

        let deferred_vao = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&Self::ARRAY),
            usage: wgpu::BufferUsages::VERTEX,
            label: Some("deferred_vao"),
        });
        resource_container.buffers[DeferredVao] =Some(deferred_vao);
    }
}
