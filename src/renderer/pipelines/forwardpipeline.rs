use super::HorizonPipeline;

use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::texture::Texture;

use wgpu::{ColorTargetState, DepthStencilState};

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
                label: Some("forward render pipeline layout"),
                push_constant_ranges: &[],
            });

        let module_descriptor = wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../../shaders/forward.wgsl"
            ))),
            label: Some("forward shader"),
        };
        let module = device.create_shader_module(&module_descriptor);

        let vbo_layout = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x2];

        let vertex_state = wgpu::VertexState {
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: (std::mem::size_of::<f32>() * 6) as wgpu::BufferAddress,
                attributes: &vbo_layout,
                step_mode: wgpu::VertexStepMode::Vertex,
            }],
            entry_point: "vs_main",
            module: &module,
        };
        let fragment_state = Some(wgpu::FragmentState {
            targets,
            module: &module,
            entry_point: if cfg!(target_arch = "wasm32") {
                "fs_main_web"
            } else {
                "fs_main"
            },
        });

        let primitive = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            strip_index_format: None,

            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: device
                .features()
                .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            ..Default::default()
        };

        RenderPipelineBuilder::create_pipeline(
            fragment_state,
            primitive,
            vertex_state,
            device,
            &render_pipeline_layout,
            Some("forward Render pipeline"),
            None,
        )
    }
}
