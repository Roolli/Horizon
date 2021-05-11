use crate::renderer::{
    pipelines::RenderPipelineBuilder,
    primitives::{
        texture::Texture,
        vertex::{ModelVertex, Vertex},
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
        let (diffuse_bind_group, uniform_bind_group) = bind_group_layouts;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&diffuse_bind_group, &uniform_bind_group],
                label: Some("GBuffer pipeline layout"),
                push_constant_ranges: &[],
            });

        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/gbuffer.vert.spv")),
            flags: wgpu::ShaderFlags::empty(),
            label: Some("Gbuffer vertex shader"),
        });
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/gbuffer.frag.spv")),
            flags: wgpu::ShaderFlags::empty(),
            label: Some("GBuffer fragment shader"),
        });

        let vertex_state = wgpu::VertexState {
            buffers: &[ModelVertex::desc()],
            entry_point: "main",
            module: &vs_module,
        };
        let fragment_state = Some(wgpu::FragmentState {
            targets,
            module: &fs_module,
            entry_point: "main",
        });

        let depth_stencil_state = wgpu::DepthStencilState {
            bias: wgpu::DepthBiasState {
                ..Default::default()
            },
            clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            depth_compare: wgpu::CompareFunction::Less,
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        };
        let primitve_state = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            strip_index_format: if cfg!(target_arch = "wasm32") {
                Some(wgpu::IndexFormat::Uint32)
            } else {
                None
            },
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };

        RenderPipelineBuilder::create_pipeline(
            fragment_state,
            primitve_state,
            vertex_state,
            &device,
            &render_pipeline_layout,
            Some("GBuffer pipeline"),
            Some(depth_stencil_state),
        )
    }
}
