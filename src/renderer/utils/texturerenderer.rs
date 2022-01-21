use wgpu::util::DeviceExt;

/// Debug renderer for textures like shadow maps
pub struct TextureRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub quad: wgpu::Buffer,
}

impl TextureRenderer {
    pub const QUAD_VERTEX_ARRAY: [[f32; 2]; 4] =
        [[-1.0, 1.0], [-1.0, -1.0], [1.0, 1.0], [1.0, -1.0]];
    pub fn new(
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        swap_chain_descriptor: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Renderer"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    visibility: wgpu::ShaderStages::FRAGMENT,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.1,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Texture_renderer bind_group"),
            layout: &bind_group_layout,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Texture Renderer layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&Self::QUAD_VERTEX_ARRAY),
            label: None,
        });
        let module =
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/textureRenderer.wgsl"));

        let attribs = &wgpu::vertex_attr_array![0=>Float32x2];
        let buffer_layout = wgpu::VertexBufferLayout {
            attributes: attribs,
            array_stride: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[buffer_layout],
                entry_point: "vs_main",
            },
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                targets: &[swap_chain_descriptor.format.into()],
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
                strip_index_format: if cfg!(target_arch = "wasm32") {
                    Some(wgpu::IndexFormat::Uint32)
                } else {
                    None
                },
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
        });

        Self {
            quad: quad_buffer,
            bind_group,
            render_pipeline: pipeline,
        }
    }
}
