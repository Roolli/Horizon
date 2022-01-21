use wgpu::{Device, TextureDescriptor};

use crate::{
    resources::bindingresourcecontainer::BindingResourceContainer,
};

pub struct GBuffer;

impl GBuffer {
    pub fn generate_g_buffers(
        device: &Device,
        sc_descriptor: &wgpu::SurfaceConfiguration,
        resource_container: &mut BindingResourceContainer,
    ) {
        let pos_diffuse_normal_texture = device.create_texture(&TextureDescriptor {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            mip_level_count: 1,
            label: Some("position_texture"),
            sample_count: 1,
            size: wgpu::Extent3d {
                depth_or_array_layers: 3,
                height: sc_descriptor.height,
                width: sc_descriptor.width,
            },
        });
        let albedo_texture = device.create_texture(&TextureDescriptor {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            mip_level_count: 1,
            label: Some("albedo_texture"),
            sample_count: 1,
            size: wgpu::Extent3d {
                depth_or_array_layers: 1,
                height: sc_descriptor.height,
                width: sc_descriptor.width,
            },
        });
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            compare: None,
            label: Some("Texture_Sampler"),

            ..Default::default()
        });

        let position_view = pos_diffuse_normal_texture.create_view(&wgpu::TextureViewDescriptor {
            array_layer_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            ..Default::default()
        });
        let normal_view = pos_diffuse_normal_texture.create_view(&wgpu::TextureViewDescriptor {
            array_layer_count: std::num::NonZeroU32::new(1),
            base_array_layer: 1,
            ..Default::default()
        });
        let albedo_view = albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());

        resource_container.textures.insert(
            String::from(stringify!(pos_diffuse_normal_texture)),
            pos_diffuse_normal_texture,
        );
        resource_container
            .textures
            .insert(String::from(stringify!(albedo_texture)), albedo_texture);

        resource_container
            .texture_views
            .insert(String::from(stringify!(albedo_view)), albedo_view);

        resource_container
            .texture_views
            .insert(String::from(stringify!(normal_view)), normal_view);

        resource_container
            .texture_views
            .insert(String::from(stringify!(position_view)), position_view);
        resource_container
            .samplers
            .insert(String::from(stringify!(texture_sampler)), texture_sampler);
    }
}
