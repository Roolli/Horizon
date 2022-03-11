use std::{num::NonZeroU8};
use std::collections::HashSet;

use anyhow::*;
use bytemuck::Contiguous;
use ddsfile::{DataFormat, DxgiFormat, FourCC};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, ImageResult};
use wgpu::util::DeviceExt;
use crate::filesystem::modelimporter::Importer;
use crate::renderer::primitives::texture::ImageLoadError::ImageParseError;
use crate::SkyboxBindGroup;


pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal: bool,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label), is_normal)
    }
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal: bool,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };


        let texture = device.create_texture_with_data(queue,&wgpu::TextureDescriptor {
            label,
            dimension: wgpu::TextureDimension::D2,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            format: if is_normal {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                wgpu::TextureFormat::Rgba8UnormSrgb
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,

        },&rgba);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::FilterMode::Nearest,
            mag_filter:wgpu::FilterMode::Linear,
            min_filter:wgpu::FilterMode::Nearest,
            anisotropy_clamp: NonZeroU8::from_integer(1),
            ..Default::default()
        });
        if label.is_some() && !label.as_ref().unwrap().contains("default")
        {
            log::info!("size:{:?} name:{:?}",texture_size,label);
        }
        Ok(Self {
            sampler,
            texture,
            view,
        })
    }
    pub fn create_default_texture_with_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 3],
        label: Option<&str>,
        is_normal: bool,
    ) -> Result<Self> {
        let mut buffer: image::RgbImage = ImageBuffer::new(256, 256);
        for (_x, _y, pixel) in buffer.enumerate_pixels_mut() {
            *pixel = image::Rgb(color);
        }

        let img = DynamicImage::ImageRgb8(buffer);
        Self::from_image(device,queue, &img, label, is_normal)
    }
    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: sc_desc.width,
            depth_or_array_layers: 1,
            height: sc_desc.height,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            mip_level_count: 1,
            sample_count: 1,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::Greater),
            lod_min_clamp: 0.1,
            lod_max_clamp: 100.0,
            label: Some("depth_texture_sampler"),
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
        }
    }
    pub fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &[u8],
        label: Option<&str>,
        is_normal: bool,
    ) -> Result<Self, Error> {
        let img = image::load_from_memory(buffer).unwrap();

        Self::from_image(device, queue, &img, label, is_normal)
    }
    pub fn load_skybox_texture(device:&wgpu::Device,queue:&wgpu::Queue,buffer:&[u8]) -> (wgpu::Texture,wgpu::TextureView)
    {
        // Use DDS formats for skybox only!

        let img = ddsfile::Dds::read(buffer).unwrap();
        let texture = device.create_texture_with_data(queue,&wgpu::TextureDescriptor{
            label: Some("skybox_texture"),
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            mip_level_count:img.get_num_mipmap_levels(),
            dimension:wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count:1,
            size:wgpu::Extent3d{
                width: img.get_width(),
                height: img.get_height(),
                depth_or_array_layers:6,
            }
        },&img.data);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor{
            label:Some("skybox_texture_view"),
            dimension:Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        (texture,texture_view)

    }
    pub fn create_image_from_gltf_texture(gltf_texture:gltf::Texture,buffer_data:&[gltf::buffer::Data]) -> Result<DynamicImage,ImageLoadError>
    {
        let image =  if let gltf::image::Source::View {view,mime_type}  =  gltf_texture.source().source() {
            let data = &buffer_data[view.buffer().index()];

                image::load_from_memory(&data.0[view.offset()..view.offset()+view.length()]).map_err(|e| ImageParseError(format!("error while parsing image: Inner error: {:?}",e)))
        }else {
            Err(ImageLoadError::InvalidSource)
        };
        image
    }
}
#[derive(Clone,Debug)]
pub enum ImageLoadError {
    InvalidSource,
    ImageParseError(String),
    UnknownError,
}
