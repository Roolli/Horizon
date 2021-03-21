use super::HorizonBindGroup;
use crate::renderer::bindgroups::BindGroupContainer;
use crate::renderer::primitives::lights::directionallight::DirectionalLightRaw;
use crate::renderer::primitives::lights::pointlight::PointLightRaw;
use crate::renderer::primitives::lights::spotlight::SpotLightRaw;
use crate::renderer::state::State;

pub struct LightBindGroup;

impl HorizonBindGroup for LightBindGroup {
    fn get_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
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
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let directional_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("directional_light_buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<DirectionalLightRaw>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
        });
        let point_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("point_light_buffer"),
            mapped_at_creation: false,

            size: (State::MAX_POINT_LIGHTS * std::mem::size_of::<PointLightRaw>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
        });

        let spot_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spot_light_buffer"),
            mapped_at_creation: false,
            size: (State::MAX_SPOT_LIGHTS * std::mem::size_of::<SpotLightRaw>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout = Self::get_layout(&device);

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
        let light_bind_group_container =
            BindGroupContainer::new(light_bind_group_layout, light_bind_group);
        light_bind_group_container.add_buffer(
            stringify!(directional_light_buffer).to_string(),
            directional_light_buffer,
        );
        light_bind_group_container
            .add_buffer(stringify!(spot_light_buffer).to_string(), spot_light_buffer);
        light_bind_group_container.add_buffer(
            stringify!(point_light_buffer).to_string(),
            point_light_buffer,
        );
        light_bind_group_container
    }
}
