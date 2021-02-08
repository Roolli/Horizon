use crate::renderer::cam::Camera;
use crate::renderer::primitives::{texture::Texture, vertex::Vertex};
use crate::{filesystem::modelimporter::Importer, renderer::model::HorizonModel};

use light::DrawLight;

use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};

use super::{
    light::{self, Light},
    model::DrawModel,
    primitives::{
        instance::{Instance, InstanceRaw},
        uniforms::Uniforms,
        vertex::ModelVertex,
    },
};
pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,

    diffuse_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,
    camera: Camera,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    obj_model: HorizonModel,
    light: light::Light,
    light_bind_group: wgpu::BindGroup,
    light_bind_group_layout: wgpu::BindGroupLayout,
    light_buffer: wgpu::Buffer,
    light_render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
}

const NUM_INSTANCES_PER_ROW: u32 = 10;

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: Some("Device descriptor"),
                },
                None,
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
            format: if cfg!(target_arch = "wasm32") {
                wgpu::TextureFormat::Bgra8Unorm
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            },
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        // INSTANCING
        const SPACE: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let pos = glm::Vec3::new(x as f32, 0.0, z as f32);
                    let rot = if pos == glm::vec3(0.0, 0.0, 0.0) {
                        glm::quat_angle_axis(f32::to_radians(0.0), &glm::vec3(0.0, 0.0, 1.0))
                    } else {
                        glm::quat_angle_axis(f32::to_radians(45.0), &pos.clone().normalize())
                    };
                    Instance::new(pos, rot)
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            usage: wgpu::BufferUsage::VERTEX,
            contents: bytemuck::cast_slice(&instance_data),
        });

        // TEXTURE

        let diffuse_bytes = include_bytes!("../../assets/happy-tree.png");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: false,
                        },
                        count: None,
                    },
                ],
                label: Some("Texture bind group layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("Diffuse bind group"),
        });

        // CAMERA
        let cam = Camera {
            eye: glm::vec3(5.0, 5.0, 5.0),
            target: glm::vec3(0.0, 0.0, 0.0),
            up: glm::vec3(0.0, 1.0, 0.0), // Unit Y vector
            aspect_ratio: sc_desc.width as f32 / sc_desc.height as f32,
            fov_y: 90.0,
            z_near: 0.1,
            z_far: 100.0,
        };

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj_matrix(&cam);
        let uniform_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                }],
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    size: None,
                    offset: 0,
                },
            }],
            layout: &uniform_bind_group_layout,
        });

        // Light
        let light = Light {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Vertex Buffer"),
            contents: bytemuck::cast_slice(&[light]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            label: None,
            entries: &[wgpu::BindGroupEntry {
                resource: wgpu::BindingResource::Buffer {
                    buffer: &light_buffer,
                    offset: 0,
                    size: None,
                },
                binding: 0,
            }],
        });

        // SHADER

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                    &light_bind_group_layout,
                ],
                label: Some("Render pipeline layout"),
                push_constant_ranges: &[],
            });

        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/shader.frag.spv"));

        let basic_pipeline = State::create_pipeline(
            &vs_module,
            &fs_module,
            &device,
            &sc_desc,
            &render_pipeline_layout,
            &[ModelVertex::desc(), InstanceRaw::desc()],
            Some(Texture::DEPTH_FORMAT),
        );
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline"),
                bind_group_layouts: &[&uniform_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });

            let light_vs =
                device.create_shader_module(&wgpu::include_spirv!("../shaders/light.vert.spv"));
            let light_fs =
                device.create_shader_module(&wgpu::include_spirv!("../shaders/light.frag.spv"));

            Self::create_pipeline(
                &light_vs,
                &light_fs,
                &device,
                &sc_desc,
                &layout,
                &[ModelVertex::desc()],
                Some(Texture::DEPTH_FORMAT),
            )
        };

        // TODO: Change to some sort of IoC container where it resolves based on current arch.
        let importer;
        #[cfg(target_arch = "wasm32")]
        {
            use crate::filesystem::webfileloader::WebFileLoader;
            importer = Importer::new(Box::new(WebFileLoader::new("http://localhost:8000")));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::filesystem::nativefileloader::Nativefileloader;
            let exe_dir = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();

            importer = Importer::new(Box::new(Nativefileloader::new(exe_dir)));
        }
        let obj_model = HorizonModel::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            &importer,
            "cube.obj",
        )
        .await
        .unwrap();

        Self {
            depth_texture: Texture::create_depth_texture(&device, &sc_desc, "depth_texture"),
            instances,
            camera: cam,
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
            diffuse_bind_group,
            diffuse_texture,
            instance_buffer,
            uniform_bind_group,
            uniform_buffer,
            obj_model,
            light,
            light_bind_group,
            light_bind_group_layout,
            light_buffer,
            light_render_pipeline,
            uniforms,
        }
    }
    fn create_pipeline(
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
        device: &wgpu::Device,
        swapchain_desc: &wgpu::SwapChainDescriptor,
        pipeline_layout: &wgpu::PipelineLayout,
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
        depth_format: Option<wgpu::TextureFormat>,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &vertex_buffer_layouts,
                module: &vs_module,
                entry_point: "main",
            },
            fragment: Some(wgpu::FragmentState {
                targets: &[swapchain_desc.format.into()],
                module: &fs_module,
                entry_point: "main",
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: wgpu::CullMode::Back,
                strip_index_format: if cfg!(target_arch = "wasm32") {
                    Some(wgpu::IndexFormat::Uint32)
                } else {
                    None
                },
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            multisample: wgpu::MultisampleState {
                ..Default::default()
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState {
                    ..Default::default()
                },
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
                depth_compare: wgpu::CompareFunction::Less,
                format,
                depth_write_enabled: true,
                stencil: wgpu::StencilState::default(),
            }),
        })
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_descriptor.height = new_size.height;
        self.sc_descriptor.width = new_size.width;
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.sc_descriptor, "depth_texture");
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
            _ => false,
        }
    }
    pub fn update(&mut self) {
        let old_light_pos: glm::Vec3 = self.light.position.into();
        self.light.position = glm::rotate_y_vec3(&old_light_pos, f32::to_radians(1.0f32)).into();
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
    }
    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass descriptor"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.uniform_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            let mesh = &self.obj_model.meshes[0]; // we assume there is at least one mesh
            let material = &self.obj_model.materials[mesh.material];
            render_pass.draw_mesh_instanced(
                mesh,
                0..self.instances.len() as u32,
                material,
                &self.uniform_bind_group,
                &self.light_bind_group,
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
