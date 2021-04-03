use super::HorizonPipeline;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::vertex::{ModelVertex, Vertex};
use specs::{Component, NullStorage};
use wgpu::BindGroupLayout;

pub struct ShadowPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for ShadowPipeline {
    type RequiredLayouts = &'a BindGroupLayout;

    fn create_pipeline(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        bind_group_layouts: Self::RequiredLayouts,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[&bind_group_layouts],
            push_constant_ranges: &[],
        });

        // [2021-04-02T16:10:08Z ERROR wgpu_core::validation] Unexpected varying type: Array { base: [1], size: Constant([5]), stride: None }
        // let vs_module =
        //     device.create_shader_module(&wgpu::include_spirv!("../../shaders/shadow.vert.spv"));
        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/shadow.vert.spv")),
            flags: wgpu::ShaderFlags::empty(),
            label: Some("shadow_vertex_shader"),
        });
        let depth_stencil_state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            bias: wgpu::DepthBiasState {
                clamp: 0.0,
                constant: 2, // bilinear filtering
                slope_scale: 2.0,
            },
            depth_compare: wgpu::CompareFunction::LessEqual,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
            clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
        };
        //TODO: create seperate vertex buffer for location only for performance reasons
        let vertex_state = wgpu::VertexState {
            buffers: &[ModelVertex::desc()],
            entry_point: "main",
            module: &vs_module,
        };
        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Front),
            strip_index_format: if cfg!(target_arch = "wasm32") {
                Some(wgpu::IndexFormat::Uint32)
            } else {
                None
            },
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };
        RenderPipelineBuilder::create_pipeline(
            None,
            primitve_state,
            vertex_state,
            &device,
            &pipeline_layout,
            Some("Shadow pipeline"),
            Some(depth_stencil_state),
        )
    }
}
