use super::HorizonPipeline;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::texture::Texture;
use crate::renderer::primitives::vertex::{ModelVertex, Vertex};
use specs::*;
use wgpu::ColorTargetState;

pub struct ForwardPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for ForwardPipeline {
    type RequiredLayouts = (
        &'a wgpu::BindGroupLayout,
        &'a wgpu::BindGroupLayout,
        &'a wgpu::BindGroupLayout,
    );

    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: Self::RequiredLayouts,
        targets: &[ColorTargetState],
    ) -> wgpu::RenderPipeline {
        let (deferred_bind_group, uniform_bind_group, light_bind_group) = bind_group_layouts;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[deferred_bind_group, uniform_bind_group, light_bind_group],
                label: Some("Render pipeline layout"),
                push_constant_ranges: &[],
            });

        let module =
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/forward.wgsl"));

        let vbo_layout = wgpu::vertex_attr_array![0=>Float32x2];

        let vertex_state = wgpu::VertexState {
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                attributes: &vbo_layout,
                step_mode: wgpu::VertexStepMode::Vertex,
            }],
            entry_point: "vs_main",
            module: &module,
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
        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            strip_index_format: None,
            //  if cfg!(target_arch = "wasm32") {
            //     Some(wgpu::IndexFormat::Uint32)
            // } else {
            //     None
            // },
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };

        RenderPipelineBuilder::create_pipeline(
            fragment_state,
            primitve_state,
            vertex_state,
            device,
            &render_pipeline_layout,
            Some("forward Render pipeline"),
            Some(depth_stencil_state),
        )
    }
}
