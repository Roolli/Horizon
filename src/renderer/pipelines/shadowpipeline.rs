use super::HorizonPipeline;

use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::vertex::{MeshVertexData, Vertex};

use wgpu::{vertex_attr_array, BindGroupLayout, ColorTargetState};

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

        let wgsl = if cfg!(target_arch = "wasm32") {
            wgpu::include_wgsl!("../../shaders/web/shadow.wgsl")
        } else {
            wgpu::include_wgsl!("../../shaders/native/shadow.wgsl")
        };
        let module = device.create_shader_module(&wgsl);
        let depth_stencil_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            bias: wgpu::DepthBiasState {
                clamp: 0.0,
                constant: 2, // bi-linear filtering
                slope_scale: 2.0,
            },
            depth_compare: wgpu::CompareFunction::LessEqual,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        };
        let attr_array = vertex_attr_array![0=>Float32x3];
        let vertex_state = wgpu::VertexState {
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<MeshVertexData>() as wgpu::BufferAddress,
                attributes: &attr_array,
                step_mode: wgpu::VertexStepMode::Vertex,
            }],
            entry_point: "vs_main",
            module: &module,
        };
        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
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
