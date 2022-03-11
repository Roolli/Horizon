use bytemuck::Contiguous;
use wgpu::SamplerBindingType;

/// Debug renderer for textures like shadow maps
pub struct TextureRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

impl TextureRenderer {
    pub fn hdr_texture_visualizer(
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        swap_chain_descriptor: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::BindGroup,wgpu::RenderPipeline) {

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Renderer"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry{
                    binding:1,
                    visibility:wgpu::ShaderStages::FRAGMENT,
                    ty:wgpu::BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count:None,
                }
            ],
        });


        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry{
                    binding:1,
                    resource:wgpu::BindingResource::Sampler(&sampler),
                }

            ],
            label: Some("Texture_renderer bind_group"),
            layout: &bind_group_layout,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Texture Renderer layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let module =
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/textureRenderer.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[],
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
                strip_index_format: None,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
        });

        (bind_group, pipeline)

    }
    pub fn new_depth_texture_visualizer( device: &wgpu::Device,
                                         texture_view: &wgpu::TextureView,
                                         swap_chain_descriptor: &wgpu::SurfaceConfiguration) ->(wgpu::BindGroup,wgpu::RenderPipeline)
    {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Depth_debug_renderer"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },

            ],
            label: Some("Depth_texture_bind_group"),
            layout: &bind_group_layout,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Depth Texture Renderer layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let module =
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/textureRenderer.wgsl"));


        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                buffers: &[],
                entry_point: "depth_vs_main",
            },
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                targets: &[swap_chain_descriptor.format.into()],
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
                strip_index_format:None,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
        });

        (bind_group,pipeline)
    }
}
