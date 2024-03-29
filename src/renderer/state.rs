use crate::renderer::primitives::texture::Texture;
use std::ops::BitAnd;

use crate::components::transform::*;
use specs::WorldExt;
use winit::{event::*, window::Window};

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_descriptor: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: Texture,
    pub scale_factor: f64,
}
impl State {
    pub const OPENGL_TO_WGPU_MATRIX: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ];
    pub const CASCADE_DISTS: (f32, f32) = (0.1, 10.0);
    pub const NUM_PASSES: u32 = 6;
    pub const MAX_ENTITY_COUNT: wgpu::BufferAddress =
        (std::mem::size_of::<TransformRaw>() * 2048) as wgpu::BufferAddress;
    pub const MAX_POINT_LIGHTS: usize = 1024;
    pub const MAX_SPOT_LIGHTS: usize = 1024;
    pub const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        depth_or_array_layers: if !cfg!(target_arch = "wasm32") { 4 } else { 1 },
        height: 1024,
        width: 1024,
    };
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                force_fallback_adapter: false,
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: adapter.features().bitand(
                        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::PIPELINE_STATISTICS_QUERY,
                    ),
                    limits: wgpu::Limits::default(),
                    label: Some("Device descriptor"),
                },
                None,
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: if cfg!(target_arch = "wasm32") {
                wgpu::TextureFormat::Bgra8Unorm
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            },
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &sc_desc);

        Self {
            depth_texture: Texture::create_depth_texture(&device, &sc_desc, "depth_texture"),
            device,
            surface,
            queue,
            sc_descriptor: sc_desc,
            size,
            scale_factor: window.scale_factor(),
        }
    }
}
