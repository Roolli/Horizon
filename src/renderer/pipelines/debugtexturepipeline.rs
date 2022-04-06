use crate::HorizonPipeline;
use wgpu::{ColorTargetState, Device, RenderPipeline};

pub struct DebugTexturePipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for DebugTexturePipeline {
    type RequiredLayouts = (&'a wgpu::BindGroupLayout);

    fn create_pipeline(
        device: &Device,
        bind_group_layouts: Self::RequiredLayouts,
        targets: &[ColorTargetState],
    ) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Texture Renderer layout"),
            bind_group_layouts: &[bind_group_layouts],
            push_constant_ranges: &[],
        });
        let module =
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/textureRenderer.wgsl"));
        let attribs = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x2];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[wgpu::VertexBufferLayout {
                    step_mode: wgpu::VertexStepMode::Vertex,
                    array_stride: (std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress),
                    attributes: &attribs,
                }],
                entry_point: "vs_main",
            },
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                targets,
                entry_point: "fs_main",
                module: &module,
            }),
            label: Some("Debug texture renderer pipeline"),
            layout: Some(&pipeline_layout),
            multisample: wgpu::MultisampleState {
                ..Default::default()
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
        });
        pipeline
    }
}
