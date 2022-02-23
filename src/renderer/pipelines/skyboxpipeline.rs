use std::borrow::Cow;
use egui::CursorIcon::Default;
use wgpu::{BindGroupLayout, ColorTargetState, Device, RenderPipeline};
use crate::HorizonPipeline;
use crate::renderer::pipelines::RenderPipelineBuilder;
use crate::renderer::primitives::texture::Texture;

pub struct SkyboxPipeline(pub wgpu::RenderPipeline);


impl<'a> HorizonPipeline<'a> for SkyboxPipeline{
    type RequiredLayouts = (&'a BindGroupLayout);

    fn create_pipeline(device: &Device, bind_group_layouts: Self::RequiredLayouts, targets: &[ColorTargetState]) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("skybox_render_pipeline_layout"),
            push_constant_ranges:&[],
            bind_group_layouts:&[bind_group_layouts],
        });
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
            label: Some("Skybox_Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders/skybox.wgsl")))
        });
        let vertex_state = wgpu::VertexState{
            buffers:&[],
            entry_point: "sky_vs",
            module:&shader_module,
        };
        let fragment_state =wgpu::FragmentState {
            module: &shader_module,
            targets,
            entry_point:"sky_fs"
        };
        let primitive_state = wgpu::PrimitiveState{
          front_face: wgpu::FrontFace::Ccw,
            ..wgpu::PrimitiveState::default()
        };
        let depth_stencil = wgpu::DepthStencilState{
            bias: wgpu::DepthBiasState::default(),
            stencil: wgpu::StencilState::default(),
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled:false,
            depth_compare: wgpu::CompareFunction::GreaterEqual,
        };

        RenderPipelineBuilder::create_pipeline(Some(fragment_state),primitive_state,vertex_state,device,&pipeline_layout,Some("Skybox render pipeline"),Some(depth_stencil))

    }
}