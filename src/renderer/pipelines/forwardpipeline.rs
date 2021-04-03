use super::HorizonPipeline;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::texture::Texture;
use crate::renderer::primitives::vertex::{ModelVertex, Vertex};
use specs::*;

pub struct ForwardPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for ForwardPipeline {
    type RequiredLayouts = (
        &'a wgpu::BindGroupLayout,
        &'a wgpu::BindGroupLayout,
        &'a wgpu::BindGroupLayout,
    );

    fn create_pipeline(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        bind_group_layouts: Self::RequiredLayouts,
    ) -> wgpu::RenderPipeline {
        let (diffuse_bind_group, uniform_bind_group, light_bind_group) = bind_group_layouts;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&diffuse_bind_group, &uniform_bind_group, &light_bind_group],
                label: Some("Render pipeline layout"),
                push_constant_ranges: &[],
            });

        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/shader.vert.spv")),
            flags: wgpu::ShaderFlags::empty(),
            label: Some("Forward vertex shader"),
        });
        // let vs_module =
        //     device.create_shader_module(&wgpu::include_spirv!("../../shaders/shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/shader.frag.spv")),
            flags: wgpu::ShaderFlags::empty(),
            label: Some("forward fragment shader"),
        });
        // let fs_module =
        //     device.create_shader_module(&wgpu::include_spirv!("../../shaders/shader.frag.spv"));
        let vertex_state = wgpu::VertexState {
            buffers: &[ModelVertex::desc()],
            entry_point: "main",
            module: &vs_module,
        };
        let view_targets = &[swap_chain_desc.format.into()];
        let fragment_state = Some(wgpu::FragmentState {
            targets: view_targets,
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
            Some("Render pipeline"),
            Some(depth_stencil_state),
        )
    }
}
