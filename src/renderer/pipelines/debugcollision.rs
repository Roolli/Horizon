use crate::HorizonPipeline;
use wgpu::{ColorTargetState, Device, RenderPipeline};

pub struct DebugCollisionPipeline(pub wgpu::RenderPipeline);

impl<'a> HorizonPipeline<'a> for DebugCollisionPipeline {
    type RequiredLayouts = (&'a wgpu::BindGroupLayout);

    fn create_pipeline(
        device: &Device,
        bind_group_layouts: Self::RequiredLayouts,
        targets: &[ColorTargetState],
    ) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("debug collisions pipeline layout"),
            bind_group_layouts: &[bind_group_layouts],
            push_constant_ranges: &[],
        });
        let module = device
            .create_shader_module(&wgpu::include_wgsl!("../../shaders/colliderRenderer.wgsl"));
        let attribs = wgpu::vertex_attr_array![0=>Float32x3];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug collision pipeline"),
            layout: Some(&pipeline_layout),
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[wgpu::VertexBufferLayout {
                    step_mode: wgpu::VertexStepMode::Vertex,
                    array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    attributes: &attribs,
                }],
                entry_point: "vs_main",
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                ..Default::default()
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Line,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &module,
                targets,
            }),
        });
        pipeline
    }
}
