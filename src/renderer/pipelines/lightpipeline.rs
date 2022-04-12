use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::vertex::{MeshVertexData, Vertex};
use wgpu::{BindGroupLayout, ColorTargetState};

use super::HorizonPipeline;

use crate::renderer::primitives::texture::Texture;

pub struct LightPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for LightPipeline {
    type RequiredLayouts = (&'a BindGroupLayout, &'a BindGroupLayout);

    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: Self::RequiredLayouts,
        targets: &[ColorTargetState],
    ) -> wgpu::RenderPipeline {
        let (uniform_bind_group, light_bind_group) = bind_group_layouts;
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline"),
            bind_group_layouts: &[uniform_bind_group, light_bind_group],
            push_constant_ranges: &[],
        });

        let wgsl = if cfg!(target_arch = "wasm32") {
            wgpu::include_wgsl!("../../shaders/web/light.wgsl")
        } else {
            wgpu::include_wgsl!("../../shaders/native/light.wgsl")
        };
        let module = device.create_shader_module(&wgsl);
        let vertex_state = wgpu::VertexState {
            buffers: &[],
            entry_point: "vs_main",
            module: &module,
        };

        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            strip_index_format: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };

        let fragment_state = Some(wgpu::FragmentState {
            targets,
            module: &module,
            entry_point: "fs_main",
        });

        let depth_stencil_state = wgpu::DepthStencilState {
            bias: wgpu::DepthBiasState {
                ..Default::default()
            },
            depth_compare: wgpu::CompareFunction::Less,
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        };

        RenderPipelineBuilder::create_pipeline(
            fragment_state,
            primitve_state,
            vertex_state,
            device,
            &layout,
            Some("Light Render pipeline"),
            Some(depth_stencil_state),
        )
    }
}
