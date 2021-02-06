use std::{ops::Range, path::Path};

use anyhow::Context;
use fileloader::FileLoader;
use image::{DynamicImage, EncodableLayout, GenericImageView, ImageBuffer, Rgb, RgbImage};
use nalgebra::Dynamic;
use wgpu::{util::DeviceExt, BindGroup, BindGroupDescriptor, IndexFormat};

use crate::filesystem::{fileloader, modelimporter::Importer};

use super::primitives::{material::Material, texture};
use super::primitives::{mesh::Mesh, vertex::ModelVertex};
pub struct HorizonModel {
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
}
impl HorizonModel {
    pub async fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        importer: &Importer,
        path: &str,
    ) -> Result<Self, anyhow::Error> {
        let (obj_models, obj_materials) = importer.import_obj_model(path).await.unwrap();

        let mut mats = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            if diffuse_path.is_empty() {
                continue;
            }
            let diffuse_texture = texture::Texture::load(
                &device,
                &queue,
                importer.import_file(diffuse_path.as_str()).await.as_slice(),
                Some(diffuse_path.as_str()),
            )?;

            let bind_group = Self::create_bind_group(&device, &layout, &diffuse_texture);
            mats.push(Material {
                diffuse_texture,
                name: mat.name,
                bind_group,
            });
        }
        // Create texture to represent missing texture.
        if mats.is_empty() {
            let mut buffer: RgbImage = ImageBuffer::new(512, 512);
            for (x, y, pixel) in buffer.enumerate_pixels_mut() {
                *pixel = image::Rgb([255, 0, 0]);
            }
            let name = format!("{}_BLANK", &path);
            let img = DynamicImage::ImageRgb8(buffer);
            let texture = texture::Texture::from_image(&device, &queue, &img, Some(name.as_str()))?;
            let bind_group = Self::create_bind_group(&device, &layout, &texture);
            mats.push(Material {
                diffuse_texture: texture,
                name,
                bind_group,
            });
        }

        let mut meshes = Vec::new();
        for model in obj_models {
            let mut verticies = Vec::new();
            assert!(
                model.mesh.positions.len() % 3 == 0,
                "positions isn't correct"
            );
            for i in 0..model.mesh.positions.len() / 3 {
                let texture_coords: [f32; 2] = if model.mesh.texcoords.is_empty() {
                    [0.0, 0.0]
                } else {
                    [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]]
                };
                let normals: [f32; 3] = if model.mesh.normals.is_empty() {
                    [0.0, 0.0, 0.0]
                } else {
                    [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ]
                };
                verticies.push(ModelVertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: texture_coords,
                    normals,
                })
            }
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex buffer", path)),
                contents: bytemuck::cast_slice(&verticies),
                usage: wgpu::BufferUsage::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                usage: wgpu::BufferUsage::INDEX,
                label: Some(&format!("{:?} Index buffer", path)),
                contents: bytemuck::cast_slice(&model.mesh.indices),
            });
            meshes.push(Mesh {
                name: model.name,
                vertex_buffer,
                index_buffer,
                element_count: model.mesh.indices.len() as u32,
                material: model.mesh.material_id.unwrap_or(0),
            });
        }

        Ok(Self {
            materials: mats,
            meshes,
        })
    }
    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        diffuse_texture: &texture::Texture,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            label: None,
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
        })
    }
}

pub trait DrawModel<'a, 'b>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        bind_group: &'b wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
    );
    fn draw_model(&mut self, model: &'b HorizonModel, uniforms: &'b wgpu::BindGroup);
    fn draw_model_instanced(
        &mut self,
        model: &'b HorizonModel,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material, uniforms: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, 0..1, material, uniforms)
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.set_bind_group(0, &material.bind_group, &[]);

        self.set_bind_group(1, &uniforms, &[]);
        self.draw_indexed(0..mesh.element_count, 0, instances);
    }

    fn draw_model(&mut self, model: &'b HorizonModel, uniforms: &'b wgpu::BindGroup) {
        self.draw_model_instanced(model, 0..1, uniforms);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b HorizonModel,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(
                mesh,
                instances.clone(),
                &model.materials[mesh.material],
                uniforms,
            );
        }
    }
}
