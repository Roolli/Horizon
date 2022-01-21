use crate::renderer::primitives::vertex::ModelVertex;
use mesh::Mesh;
use nalgebra::Point3;
use wgpu::util::DeviceExt;

use crate::filesystem::modelimporter::Importer;

use super::{
    model::HorizonModel,
    primitives::{material::Material, mesh, texture},
};

pub struct ModelBuilder {
    pub diffuse_texture_bind_group_layout: wgpu::BindGroupLayout,
    importer: Importer,
}

impl ModelBuilder {
    pub fn new(device: &wgpu::Device, importer: Importer) -> Self {
        Self {
            diffuse_texture_bind_group_layout: Self::get_diffuse_texture_bind_group_layout(&device),
            importer,
        }
    }
    fn get_diffuse_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("diffuse texture bind group layout"),
        })
    }
    pub async fn create(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> HorizonModel {
        let (obj_models, obj_materials) = self.importer.import_obj_model(path).await.unwrap();

        let mut mats = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            if diffuse_path.is_empty() {
                continue;
            }
            let diffuse_texture = texture::Texture::load(
                &device,
                &queue,
                self.importer
                    .import_file(diffuse_path.as_str())
                    .await
                    .as_slice(),
                Some(diffuse_path.as_str()),
                false,
            )
            .unwrap();
            let normal_texture = if !(mat.normal_texture.is_empty()) {
                texture::Texture::load(
                    &device,
                    &queue,
                    self.importer
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

            let bind_group = Self::create_bind_group(
                &device,
                &self.diffuse_texture_bind_group_layout,
                &diffuse_texture,
                &normal_texture,
            );
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
            let bind_group = Self::create_bind_group(
                &device,
                &self.diffuse_texture_bind_group_layout,
                &diffuse_texture,
                &normal_texture,
            );
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
            let _min_extents = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
            let _max_extents = glm::vec3(f32::MIN, f32::MIN, f32::MIN);
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
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                usage: wgpu::BufferUsages::INDEX,
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

        HorizonModel {
            materials: mats,
            meshes,
        }
    }
    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        diffuse_texture: &texture::Texture,
        normal_texture: &texture::Texture,
    ) -> wgpu::BindGroup {
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
