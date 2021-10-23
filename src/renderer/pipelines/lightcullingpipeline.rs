use wgpu::BindGroupLayout;

use super::HorizonComputePipeline;

pub struct LightCullingPipeline(pub wgpu::ComputePipeline);

impl<'a> HorizonComputePipeline<'a> for LightCullingPipeline {
    type RequiredLayouts = (
        &'a BindGroupLayout,
        &'a BindGroupLayout,
        &'a BindGroupLayout,
    );

    fn create_compute_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: Self::RequiredLayouts,
    ) -> wgpu::ComputePipeline {
        let (uniform_bind_group, light_bind_group, tile_bind_group) = &bind_group_layouts;
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[light_bind_group, uniform_bind_group, tile_bind_group],
            label: Some("Light Culling Pipeline Layout"),
            push_constant_ranges: &[],
        });
        let module =
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/lightculling.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            entry_point: "main",
            label: Some("Light Culling pipeline"),
            layout: Some(&layout),
            module: &module,
        })
    }
}
