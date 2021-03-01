use std::ops::Range;

use glm::vec3;
use image::{DynamicImage, ImageBuffer};

use nalgebra::{Norm, Point3};
use wgpu::{util::DeviceExt, BindGroup};

use crate::filesystem::modelimporter::Importer;

use super::primitives::{material::Material, texture};
use super::primitives::{mesh::Mesh, vertex::ModelVertex};
use specs::{Component, VecStorage};
#[derive(Component)]
#[storage(VecStorage)]
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
                false,
            )?;
            let normal_texture = if !(mat.normal_texture.is_empty()) {
                texture::Texture::load(
                    &device,
                    &queue,
                    importer
                        .import_file(mat.normal_texture.as_str())
                        .await
                        .as_slice(),
                    Some("normal_texture"),
                    false,
                )
                .unwrap()
            } else {
                texture::Texture::create_default_texture_with_color(
                    &device,
                    &queue,
                    [0, 0, 255],
                    Some("default_normal_texture"),
                    true,
                )
            };

            let bind_group =
                Self::create_bind_group(&device, &layout, &diffuse_texture, &normal_texture);
            mats.push(Material {
                diffuse_texture,
                name: mat.name,
                bind_group,
                normal_texture,
            });
        }
        // Create texture to represent missing texture.
        if mats.is_empty() {
            let diffuse_texture = texture::Texture::create_default_texture_with_color(
                &device,
                &queue,
                [255, 0, 0],
                Some("DEFAULT_DIFFUSE_TEXTURE"),
                false,
            );
            let normal_texture = texture::Texture::create_default_texture_with_color(
                &device,
                &queue,
                [0, 0, 255],
                Some("DEFAULT_NORMAL_TEXTURE"),
                true,
            );
            let bind_group =
                Self::create_bind_group(&device, &layout, &diffuse_texture, &normal_texture);
            mats.push(Material {
                diffuse_texture,
                name: String::from("DEFAULT_MATERIAL"),
                bind_group,
                normal_texture,
            });
        }

        let mut meshes = Vec::new();
        for model in obj_models {
            let mut verticies = Vec::new();
            assert!(
                model.mesh.positions.len() % 3 == 0,
                "position layout is wrong"
            );
            let mut min_extents = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
            let mut max_extents = glm::vec3(f32::MIN, f32::MIN, f32::MIN);
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
                let position = [
                    model.mesh.positions[i * 3],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ];
                //Self::update_bounding_box_extents(&mut min_extents, &mut max_extents, position);

                verticies.push(ModelVertex {
                    position,
                    tex_coords: texture_coords,
                    normals,
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
            }
            let indices = &model.mesh.indices;
            for chunk in indices.chunks(3) {
                let v0 = verticies[chunk[0] as usize];
                let v1 = verticies[chunk[1] as usize];
                let v2 = verticies[chunk[2] as usize];

                let pos0: glm::Vec3 = v0.position.into();
                let pos1: glm::Vec3 = v1.position.into();
                let pos2: glm::Vec3 = v2.position.into();

                let uv0: glm::Vec2 = v0.tex_coords.into();
                let uv1: glm::Vec2 = v1.tex_coords.into();
                let uv2: glm::Vec2 = v2.tex_coords.into();

                // Triangle edges
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;
                // Maths stuff:
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Solution from: https://sotrh.github.io/learn-wgpu/intermediate/tutorial11-normals/#the-tangent-and-the-bitangent
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;
                verticies[chunk[0] as usize].tangent = tangent.into();
                verticies[chunk[1] as usize].tangent = tangent.into();
                verticies[chunk[2] as usize].tangent = tangent.into();

                verticies[chunk[0] as usize].bitangent = bitangent.into();
                verticies[chunk[1] as usize].bitangent = bitangent.into();
                verticies[chunk[2] as usize].bitangent = bitangent.into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex buffer", path)),
                contents: bytemuck::cast_slice(&verticies),
                usage: wgpu::BufferUsage::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                usage: wgpu::BufferUsage::INDEX,
                label: Some(&format!("{} Index buffer", path)),
                contents: bytemuck::cast_slice(&model.mesh.indices),
            });
            meshes.push(Mesh {
                points: verticies
                    .iter()
                    .map(|v| Point3::new(v.position[0], v.position[1], v.position[2]))
                    .collect::<Vec<_>>(),
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
        normal_texture: &texture::Texture,
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
        })
    }
    fn update_bounding_box_extents(
        min_extent: &mut glm::Vec3,
        max_extent: &mut glm::Vec3,
        coords: [f32; 3],
    ) {
        min_extent.x = f32::min(min_extent.x, coords[0]);
        min_extent.y = f32::min(min_extent.y, coords[1]);
        min_extent.z = f32::min(min_extent.x, coords[2]);

        max_extent.x = f32::max(max_extent.x, coords[0]);
        max_extent.y = f32::max(max_extent.y, coords[1]);
        max_extent.z = f32::max(max_extent.x, coords[2]);
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
        light: &'b wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_model(
        &mut self,
        model: &'b HorizonModel,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'b HorizonModel,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, 0..1, material, uniforms, light)
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.set_bind_group(0, &material.bind_group, &[]);

        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.draw_indexed(0..mesh.element_count, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b HorizonModel,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, 0..1, uniforms, light);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b HorizonModel,
        instances: Range<u32>,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(
                mesh,
                instances.clone(),
                &model.materials[mesh.material],
                uniforms,
                light,
            );
        }
    }
}
