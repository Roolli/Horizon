use super::HorizonBindGroup;
use crate::renderer::{
    bindgroups::BindGroupContainer,
    primitives::lights::{
        directionallight::DirectionalLightRaw, pointlight::PointLightRaw, spotlight::SpotLightRaw,
    },
    state::State,
};
use specs::*;
use crate::{BufferTypes};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct LightBindGroup;

impl<'a> HorizonBindGroup<'a> for LightBindGroup {
    type BindingResources = (&'a wgpu::Buffer, &'a wgpu::Buffer, &'a wgpu::Buffer);
    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,

                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT
                        | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT
                        | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("light_bind_group_layout"),
        })
    }

    fn create_container(
        device: &wgpu::Device,
        binding_resources: Self::BindingResources,
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let (directional_light_buffer, point_light_buffer, spot_light_buffer) = binding_resources;

        let light_bind_group_layout = Self::get_layout(device);

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    resource: directional_light_buffer.as_entire_binding(),
                    binding: 0,
                },
                wgpu::BindGroupEntry {
                    resource: point_light_buffer.as_entire_binding(),
                    binding: 1,
                },
                wgpu::BindGroupEntry {
                    resource: spot_light_buffer.as_entire_binding(),
                    binding: 2,
                },
            ],
        });

        BindGroupContainer::new(light_bind_group_layout, light_bind_group)
    }

    fn get_resources(
        device: &wgpu::Device,
        resource_container: &mut crate::resources::bindingresourcecontainer::BindingResourceContainer,
    ) {
        let directional_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("directional_light_buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<DirectionalLightRaw>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let point_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("point_light_buffer"),
            mapped_at_creation: false,

            size: (State::MAX_POINT_LIGHTS * std::mem::size_of::<PointLightRaw>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });

        let spot_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spot_light_buffer"),
            mapped_at_creation: false,
            size: (State::MAX_SPOT_LIGHTS * std::mem::size_of::<SpotLightRaw>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        resource_container.buffers[BufferTypes::DirectionalLight]=Some(directional_light_buffer);
        resource_container
            .buffers[BufferTypes::SpotLight]
            = Some(spot_light_buffer);
        resource_container
            .buffers[BufferTypes::PointLight] =Some(point_light_buffer);
    }
}
