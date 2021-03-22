use crate::renderer::pipelines::RenderPipelineContainer;
use crate::renderer::primitives::vertex::{ModelVertex, Vertex};
use wgpu::{BindGroupLayout, ShaderFlags};

use super::HorizonPipeline;
use crate::renderer::bindgroupcontainer::BindGroupContainer;
use crate::renderer::primitives::texture::Texture;

use specs::{Component, NullStorage};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct LightPipeline;

impl<'a> HorizonPipeline<'a> for LightPipeline {
    type RequiredLayouts = (&'a BindGroupLayout, &'a BindGroupLayout);

    fn create_pipeline(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        bind_group_layouts: Self::RequiredLayouts,
    ) -> super::RenderPipelineContainer {
        let (uniform_bind_group, light_bind_group) = bind_group_layouts;
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline"),
            bind_group_layouts: &[&uniform_bind_group, &light_bind_group],
            push_constant_ranges: &[],
        });
        // https://github.com/gfx-rs/naga/issues/406 have to disable validation
        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(include_bytes!("../../shaders/light.vert.spv")),
            flags: ShaderFlags::empty(),
            label: Some("light_vertex_shader"),
        });
        let vertex_state = wgpu::VertexState {
            buffers: &[ModelVertex::desc()],
            entry_point: "main",
            module: &vs_module,
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
        let light_fs =
            device.create_shader_module(&wgpu::include_spirv!("../../shaders/light.frag.spv"));
        let view_targets = &[swap_chain_desc.format.into()];
        let fragment_state = Some(wgpu::FragmentState {
            targets: view_targets,
            module: &light_fs,
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

        RenderPipelineContainer::create_pipeline(
            fragment_state,
            primitve_state,
            vertex_state,
            &device,
            &layout,
            Some("Light Render pipeline"),
            Some(depth_stencil_state),
        )
    }
}
