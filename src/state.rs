use core::f64;

use wgpu::{util::DeviceExt, BufferUsage, VertexBufferDescriptor};
use winit::{event::*, window::Window};

use crate::primitives::vertex::Vertex;
use bytemuck;
pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    use_complex_shader: bool,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indicies_count: u32,
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_module =
            device.create_shader_module(wgpu::include_spirv!("./shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(wgpu::include_spirv!("./shaders/shader.frag.spv"));

        let basic_pipeline = State::create_pipeline(&vs_module, &fs_module, &device, &sc_desc);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer Descriptor"),
            usage: BufferUsage::VERTEX,
            contents: bytemuck::cast_slice(VERTICES),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer Descriptor"),
            usage: wgpu::BufferUsage::INDEX,
            contents: bytemuck::cast_slice(INDICES),
        });

        Self {
            device,
            surface,
            queue,
            sc_descriptor: sc_desc,
            swap_chain,
            size,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            render_pipeline: basic_pipeline,
            vertex_buffer,
            use_complex_shader: false,
            indicies_count: INDICES.len() as u32,
            index_buffer,
        }
    }
    fn create_pipeline(
        vertex_module: &wgpu::ShaderModule,
        fragment_module: &wgpu::ShaderModule,
        device: &wgpu::Device,
        swapchain_desc: &wgpu::SwapChainDescriptor,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
                label: Some("Render pipeline layout"),
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fragment_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: swapchain_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_descriptor.height = new_size.height;
        self.sc_descriptor.width = new_size.width;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.sc_descriptor);
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.y / (self.size.height as f64),
                    g: position.x / (self.size.width as f64),
                    b: 1.0,
                    a: 1.0,
                };

                true
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                } => {
                    if !self.use_complex_shader {
                        let vs_module = self.device.create_shader_module(wgpu::include_spirv!(
                            "./shaders/shader_complex.vert.spv"
                        ));
                        let fs_module = self.device.create_shader_module(wgpu::include_spirv!(
                            "./shaders/shader_complex.frag.spv"
                        ));
                        self.render_pipeline = State::create_pipeline(
                            &vs_module,
                            &fs_module,
                            &self.device,
                            &self.sc_descriptor,
                        )
                    } else {
                        let vs_module = self.device.create_shader_module(wgpu::include_spirv!(
                            "./shaders/shader.vert.spv"
                        ));
                        let fs_module = self.device.create_shader_module(wgpu::include_spirv!(
                            "./shaders/shader.frag.spv"
                        ));
                        self.render_pipeline = State::create_pipeline(
                            &vs_module,
                            &fs_module,
                            &self.device,
                            &self.sc_descriptor,
                        )
                    }
                    self.use_complex_shader = !self.use_complex_shader;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
    pub fn update(&mut self) {}
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            render_pass.draw_indexed(0..self.indicies_count, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
