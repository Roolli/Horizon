use crate::components::gltfmodel::{GltfModel, GltfPrimitive};
use crate::renderer::primitives::vertex::MeshVertexData;
use mesh::Mesh;
use rapier3d::na::{Vector2, Vector3};
use std::collections::{HashMap, HashSet};
use gltf::Document;
use tobj::{Material, Model};
use wgpu::util::DeviceExt;

use crate::filesystem::modelimporter::Importer;
use crate::renderer::primitives::material::GltfMaterial;
use crate::renderer::primitives::mesh::{GltfMesh, VertexAttributeType, VertexAttribValues};
use crate::renderer::primitives::texture::ImageLoadError;
use crate::Texture;

use super::{
    model::HorizonModel,
    primitives::{mesh, texture},
};
/// Model importer
pub struct ModelBuilder;

impl ModelBuilder {
    // pub fn create(
    //     &self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     model_data: (Vec<Model>, Vec<(Vec<u8>, Vec<u8>, String)>),
    //     name: &str,
    // ) -> HorizonModel {
    //     let (obj_models, obj_materials) = model_data;
    //     // mats
    //     let mut mats = Vec::new();
    //     for material_textures in obj_materials {
    //         let diffuse_texture = if !material_textures.0.is_empty() {
    //             crate::renderer::primitives::texture::Texture::load(
    //                 device,
    //                 queue,
    //                 material_textures.0.as_slice(),
    //                 Some(format!("diffuse-{}", material_textures.2).as_str()),
    //                 false,
    //             )
    //             .unwrap()
    //         } else {
    //             texture::Texture::create_default_texture_with_color(
    //                 device,
    //                 queue,
    //                 [255, 0, 0],
    //                 Some("DEFAULT_DIFFUSE_TEXTURE"),
    //                 false,
    //             )
    //         };
    //         let normal_texture = if !material_textures.1.is_empty() {
    //             texture::Texture::load(
    //                 device,
    //                 queue,
    //                 material_textures.1.as_slice(),
    //                 Some(format!("normal-{}", material_textures.2).as_str()),
    //                 true,
    //             )
    //             .unwrap()
    //         } else {
    //             texture::Texture::create_default_texture_with_color(
    //                 device,
    //                 queue,
    //                 [0, 0, 255],
    //                 Some("DEFAULT_NORMAL_TEXTURE"),
    //                 true,
    //             )
    //         };
    //         let bind_group = Self::create_bind_group(
    //             device,
    //             &self.diffuse_texture_bind_group_layout,
    //             &diffuse_texture,
    //             &normal_texture,
    //         );
    //         mats.push(crate::renderer::primitives::material::Material {
    //             diffuse_texture,
    //             name: material_textures.2,
    //             bind_group,
    //             normal_texture,
    //         });
    //     }
    //
    //     let mut meshes = Vec::new();
    //     for model in obj_models {
    //         let mut vertices = Vec::new();
    //         assert_eq!(
    //             model.mesh.positions.len() % 3,
    //             0,
    //             "position layout is wrong"
    //         );
    //         let _min_extents = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    //         let _max_extents = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    //         for i in 0..model.mesh.positions.len() / 3 {
    //             let texture_coords: [f32; 2] = if model.mesh.texcoords.is_empty() {
    //                 [0.0, 0.0]
    //             } else {
    //                 [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]]
    //             };
    //             let normals: [f32; 3] = if model.mesh.normals.is_empty() {
    //                 [0.0, 0.0, 0.0]
    //             } else {
    //                 [
    //                     model.mesh.normals[i * 3],
    //                     model.mesh.normals[i * 3 + 1],
    //                     model.mesh.normals[i * 3 + 2],
    //                 ]
    //             };
    //             let position = [
    //                 model.mesh.positions[i * 3],
    //                 model.mesh.positions[i * 3 + 1],
    //                 model.mesh.positions[i * 3 + 2],
    //             ];
    //             //Self::update_bounding_box_extents(&mut min_extents, &mut max_extents, position);
    //
    //             vertices.push(MeshVertexData {
    //                 position,
    //                 tex_coords: texture_coords,
    //                 normals,
    //                 tangent: [0.0; 3],
    //                 bitangent: [0.0; 3],
    //             })
    //         }
    //         let indices = &model.mesh.indices;
    //         for chunk in indices.chunks(3) {
    //             let v0 = vertices[chunk[0] as usize];
    //             let v1 = vertices[chunk[1] as usize];
    //             let v2 = vertices[chunk[2] as usize];
    //
    //             let pos0: Vector3<f32> = v0.position.into();
    //             let pos1: Vector3<f32> = v1.position.into();
    //             let pos2: Vector3<f32> = v2.position.into();
    //
    //             let uv0: Vector2<f32> = v0.tex_coords.into();
    //             let uv1: Vector2<f32> = v1.tex_coords.into();
    //             let uv2: Vector2<f32> = v2.tex_coords.into();
    //
    //             // Triangle edges
    //             let edge1 = pos1 - pos0;
    //             let edge2 = pos2 - pos0;
    //
    //             let delta_uv1 = uv1 - uv0;
    //             let delta_uv2 = uv2 - uv0;
    //             // Maths stuff:
    //             //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
    //             //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
    //             // Solution from: https://sotrh.github.io/learn-wgpu/intermediate/tutorial11-normals/#the-tangent-and-the-bitangent
    //             let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
    //             let tangent = Vector3::new(
    //                 r * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
    //                 r * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
    //                 r * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
    //             );
    //             //let tangent = (edge1 * delta_uv2.y - edge2 * delta_uv1.y) * r;
    //             //let bitangent = (edge2 * delta_uv1.x - edge1 * delta_uv2.x) * r;
    //             let bitangent = Vector3::new(
    //                 r * (-delta_uv2.x * edge1.x - delta_uv1.x * edge2.x),
    //                 r * (-delta_uv2.x * edge1.y - delta_uv1.x * edge2.y),
    //                 r * (-delta_uv2.x * edge1.z - delta_uv1.x * edge2.z),
    //             );
    //             vertices[chunk[0] as usize].tangent = tangent.into();
    //             vertices[chunk[1] as usize].tangent = tangent.into();
    //             vertices[chunk[2] as usize].tangent = tangent.into();
    //
    //             vertices[chunk[0] as usize].bitangent = bitangent.into();
    //             vertices[chunk[1] as usize].bitangent = bitangent.into();
    //             vertices[chunk[2] as usize].bitangent = bitangent.into();
    //         }
    //
    //         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{} Vertex buffer", name)),
    //             contents: bytemuck::cast_slice(&vertices),
    //             usage: wgpu::BufferUsages::VERTEX,
    //         });
    //         let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             usage: wgpu::BufferUsages::INDEX,
    //             label: Some(&format!("{} Index buffer", name)),
    //             contents: bytemuck::cast_slice(&model.mesh.indices),
    //         });
    //         meshes.push(Mesh {
    //             points: vertices
    //                 .iter()
    //                 .map(|v| rapier3d::na::Point3::new(v.position[0], v.position[1], v.position[2]))
    //                 .collect::<Vec<_>>(),
    //             name: model.name,
    //             vertex_buffer,
    //             index_buffer,
    //             element_count: model.mesh.indices.len() as u32,
    //             material: model.mesh.material_id.unwrap_or(0),
    //         });
    //     }
    //
    //     HorizonModel {
    //         materials: mats,
    //         meshes,
    //     }
    // }
    pub fn create_gltf_model(
        data: (Document,Vec<gltf::buffer::Data>,Vec<gltf::image::Data>)
    ) -> Result<HorizonModel,GltfLoadError> {
        let mut materials = Vec::new();
       // let mut named_materials = HashMap::default();
        let mut linear_textures = HashSet::default();

        for material in data.0.materials() {
            let loaded_mat = Self::load_gltf_material(&material,&data.1,&data.2,&linear_textures).map_err(|e|GltfLoadError::InnerError(format!("Error occurred while loading gltf model: Inner Error: {:?}",e)))?;
            materials.push(loaded_mat);
            // if let Some(name) = material.name() {
            //     named_materials.insert(name.to_string(), loaded_mat);
            // }
            if let Some(normal_texture) = material.normal_texture() {
                linear_textures.insert(normal_texture.texture().index());
            }
            if let Some(occlusion_texture) = material.occlusion_texture() {
                linear_textures.insert(occlusion_texture.texture().index());
            }
            if let Some(roughness_texture) = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
            {
                linear_textures.insert(roughness_texture.texture().index());
            }
        }
        let mut meshes = Vec::new();
        for mesh in data.0.meshes() {
            let mut primitives = Vec::new();
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&data.1[buffer.index()]));

                let mut mesh = GltfMesh::new(primitive.mode());
                if let Some(position_vertex_attribs) = reader.read_positions().map(|v| VertexAttribValues::Float32x3( v.collect()))
                {
                    mesh.add_vertex_attribute(
                        VertexAttributeType::Position,
                        position_vertex_attribs,
                    );
                }
                if let Some(normal_vertex_attribs) = reader.read_normals().map(|v| VertexAttribValues::Float32x3(v.collect())) {
                    mesh.add_vertex_attribute(VertexAttributeType::Normal, normal_vertex_attribs);
                }
                if let Some(tangent_vertex_attribs) = reader.read_tangents().map(|v| VertexAttribValues::Float32x4(v.collect())) {
                    mesh.add_vertex_attribute(VertexAttributeType::Tangent, tangent_vertex_attribs);
                }
                if let Some(tex_coords_attribs) =
                    reader.read_tex_coords(0).map(|v| VertexAttribValues::Float32x2(v.into_f32().collect()))
                {
                    mesh.add_vertex_attribute(
                        VertexAttributeType::TextureCoords,
                        tex_coords_attribs,
                    );
                }
                if let Some(indices) = reader.read_indices().map(|v| v.into_u32().collect()) {
                    mesh.add_indices(indices);
                }
                primitives.push(GltfPrimitive {
                    mesh,
                    material: primitive
                        .material()
                        .index(),
                });
            }
            meshes.push( GltfModel{
                primitives,
            });
        }
        Ok(HorizonModel{
            meshes,
            materials,
        })
    }

    fn update_bounding_box_extents(
        min_extent: &mut Vector3<f32>,
        max_extent: &mut Vector3<f32>,
        coords: [f32; 3],
    ) {
        min_extent.x = f32::min(min_extent.x, coords[0]);
        min_extent.y = f32::min(min_extent.y, coords[1]);
        min_extent.z = f32::min(min_extent.x, coords[2]);

        max_extent.x = f32::max(max_extent.x, coords[0]);
        max_extent.y = f32::max(max_extent.y, coords[1]);
        max_extent.z = f32::max(max_extent.x, coords[2]);
    }
    fn load_gltf_material(material: &gltf::material::Material,buffer_data:&[gltf::buffer::Data],image_data:&[gltf::image::Data],linear_textures: &HashSet<usize>) -> Result<GltfMaterial,ImageLoadError> {
        let pbr = material.pbr_metallic_roughness();

        let color = pbr.base_color_factor();

        let base_color_texture = if let Some(tex_info) = pbr.base_color_texture() {
            let res = Texture::create_image_from_gltf_texture(tex_info.texture(),buffer_data)?;
                Some(res)
        }else {
            None
        };
        let normal_map_texture = if let Some(normal_tex) = material.normal_texture() {
            let res = Texture::create_image_from_gltf_texture(normal_tex.texture(),buffer_data)?;
            Some(res)
        }else {None};

        let metallic_roughness_texture =
            if let Some(metallic_roughness) = pbr.metallic_roughness_texture() {
                let res = Texture::create_image_from_gltf_texture(metallic_roughness.texture(),buffer_data)?;
                Some(res)
            }else {
                None
            };
        let occlusion_texture = if let Some(occulsion_texture) = material.occlusion_texture() {
            let res = Texture::create_image_from_gltf_texture(occulsion_texture.texture(),buffer_data)?;
            Some(res)
        }else {
            None
        };
        let emissive = material.emissive_factor();
        let emissive_texture = if let Some(emissive_info) = material.emissive_texture() {
            let res = Texture::create_image_from_gltf_texture(emissive_info.texture(),buffer_data)?;
            Some(res)
        }else {
            None
        };
        Ok(GltfMaterial{
            base_color_texture,
            emissive_texture,
            normal_map_texture,
            occlusion_texture,
            roughness_texture:metallic_roughness_texture,
            base_color:color,
            double_sided:material.double_sided(),
            pbr_roughness:pbr.roughness_factor(),
            unlit:false,
            emissive_color:emissive,
            metallic_factor:pbr.metallic_factor(),
        })
    }
}
#[derive(Clone,Debug)]
pub enum GltfLoadError
{
    InnerError(String),
}
