use crate::renderer::bindgroups::lighting::LightBindGroup;
use crate::renderer::bindgroups::BindGroupContainer;
use crate::renderer::primitives::lights::directionallight::DirectionalLightRaw;
use crate::renderer::primitives::lights::pointlight::PointLightRaw;
use crate::renderer::primitives::lights::spotlight::SpotLightRaw;
use crate::renderer::state::State;

use super::HorizonBindGroup;

pub struct ShadowBindGroup;

impl HorizonBindGroup for ShadowBindGroup {
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
    ) -> crate::renderer::bindgroupcontainer::BindGroupContainer {
        let shadow_bind_group_layout = Self::get_layout(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[&shadow_bind_group_layout],
            push_constant_ranges: &[],
        });
        let shadow_uniforms_size = std::mem::size_of::<ShadowUniforms>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: shadow_uniforms_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
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
}
