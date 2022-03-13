use crate::renderer::{
    pipelines::RenderPipelineBuilder,
    primitives::{
        texture::Texture,
        vertex::{MeshVertexData, Vertex},
    },
};

use super::HorizonPipeline;

pub struct GBufferPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for GBufferPipeline {
    type RequiredLayouts = (&'a wgpu::BindGroupLayout, &'a wgpu::BindGroupLayout);
    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: Self::RequiredLayouts,
        targets: &[wgpu::ColorTargetState],
    ) -> wgpu::RenderPipeline {
        let (material_bind_group, global_uniforms_bind_group) = bind_group_layouts;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[global_uniforms_bind_group,material_bind_group],
                label: Some("GBuffer pipeline layout"),
                push_constant_ranges: &[],
            });
        let module_descriptor = wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../../shaders/gbuffer.wgsl"
            ))),
            label: Some("GBuffer shader"),
        };
        let module = device.create_shader_module(&module_descriptor);
        let vertex_state = wgpu::VertexState {
            buffers: &[MeshVertexData::desc()],
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
                clamp: 0.0,
                constant: -2, // bilinear filtering
                slope_scale: -2.0,
            },
            depth_compare: wgpu::CompareFunction::GreaterEqual,
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        };
        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            strip_index_format: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: device
                .features()
                .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            ..Default::default()
        };

        RenderPipelineBuilder::create_pipeline(
            fragment_state,
            primitve_state,
            vertex_state,
            device,
            &render_pipeline_layout,
            Some("GBuffer pipeline"),
            Some(depth_stencil_state),
        )
    }
}
