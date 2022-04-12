use wgpu::SamplerBindingType;

/// Debug renderer for textures like shadow maps
pub struct TextureRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

impl TextureRenderer {
    pub fn new_depth_texture_visualizer(
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        swap_chain_descriptor: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::BindGroup, wgpu::RenderPipeline) {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Depth_debug_renderer"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            }],
            label: Some("Depth_texture_bind_group"),
            layout: &bind_group_layout,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Depth Texture Renderer layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let wgsl = if cfg!(target_arch = "wasm32") {
            wgpu::include_wgsl!("../../shaders/web/textureRenderer.wgsl")
        } else {
            wgpu::include_wgsl!("../../shaders/native/textureRenderer.wgsl")
        };
        let module = device.create_shader_module(&wgsl);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[],
                entry_point: "depth_vs_main",
            },
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                targets: &[wgpu::TextureFormat::Bgra8Unorm.into()],
                entry_point: "depth_fs_main",
                module: &module,
            }),
            label: Some("Depth texture renderer pipeline"),
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

        (bind_group, pipeline)
    }
}
