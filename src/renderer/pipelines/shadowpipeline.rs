use super::HorizonPipeline;

use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::vertex::{ModelVertex, Vertex};

use wgpu::{BindGroupLayout, ColorTargetState};

pub struct ShadowPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for ShadowPipeline {
    type RequiredLayouts = &'a BindGroupLayout;

    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: Self::RequiredLayouts,
        _targets: &[ColorTargetState],
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[bind_group_layouts],
            push_constant_ranges: &[],
        });

        // [2021-04-02T16:10:08Z ERROR wgpu_core::validation] Unexpected varying type: Array { base: [1], size: Constant([5]), stride: None }
        // let vs_module =
        //     device.create_shader_module(&wgpu::include_spirv!("../../shaders/shadow.vert.spv"));
        let module_descriptor = wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../../shaders/shadow.wgsl"
            ))),
            label: Some("shadow shader"),
        };
        let module = device.create_shader_module(&module_descriptor);
        let depth_stencil_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            bias: wgpu::DepthBiasState {
                clamp: 0.0,
                constant: -2, // bilinear filtering
                slope_scale: -2.0,
            },
            depth_compare: wgpu::CompareFunction::GreaterEqual,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        };
        let vertex_state = wgpu::VertexState {
            buffers: &[ModelVertex::desc()],
            entry_point: "vs_main",
            module: &module,
        };
        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Front),
            strip_index_format: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };
        RenderPipelineBuilder::create_pipeline(
            None,
            primitve_state,
            vertex_state,
            device,
            &pipeline_layout,
            Some("Shadow pipeline"),
            Some(depth_stencil_state),
        )
    }
}
