use super::HorizonPipeline;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::pipelines::RenderPipelineContainer;
use crate::renderer::primitives::texture::Texture;
use crate::renderer::primitives::vertex::{ModelVertex, Vertex};
use specs::*;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct ForwardPipeline;

impl<'a> HorizonPipeline<'a> for ForwardPipeline {
    type RequiredLayouts = (
        &'a BindGroupContainer,
        &'a BindGroupContainer,
        &'a BindGroupContainer,
    );

    fn create_pipeline(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        bind_group_layouts: Self::RequiredLayouts,
    ) -> super::RenderPipelineContainer {
        let (diffuse_bind_group, uniform_bind_group, light_bind_group) = bind_group_layouts;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &diffuse_bind_group.layout,
                    &uniform_bind_group.layout,
                    &light_bind_group.layout,
                ],
                label: Some("Render pipeline layout"),
                push_constant_ranges: &[],
            });

        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("../../shaders/shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/shader.frag.spv")),
            flags: wgpu::ShaderFlags::empty(),
            label: Some("forward fragment shader"),
        });
        let vertex_state = wgpu::VertexState {
            buffers: &[ModelVertex::desc()],
            entry_point: "main",
            module: &vs_module,
        };
        let fragment_state = Some(wgpu::FragmentState {
            targets: &[swap_chain_desc.format.into()],
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

        RenderPipelineContainer::create_pipeline(
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
