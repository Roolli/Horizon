use crate::components::gltfmodel::{GltfModel, GltfPrimitive};
use gltf::Document;
use image::{load, DynamicImage};
use rapier3d::na::{Vector2, Vector3};
use std::collections::{HashMap, HashSet};

use crate::renderer::primitives::material::GltfMaterial;
use crate::renderer::primitives::mesh::{GltfMesh, VertexAttribValues, VertexAttributeType};
use crate::renderer::primitives::texture::ImageLoadError;
use crate::Texture;

use super::{
    model::HorizonModel,
    primitives::{mesh, texture},
};
/// Model importer
pub struct ModelBuilder;

impl ModelBuilder {
    pub fn create_gltf_model(
        data: (Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>),
    ) -> Result<HorizonModel, GltfLoadError> {
        let mut materials = HashMap::new();
        let mut loaded_textures: HashMap<usize, DynamicImage> = HashMap::new();
        for material in data.0.materials() {
            let loaded_mat = Self::load_gltf_material(&material, &data.1, &mut loaded_textures)
                .map_err(|e| {
                    GltfLoadError::InnerError(format!(
                        "Error occurred while loading gltf model: Inner Error: {:?}",
                        e
                    ))
                })?;
            materials.insert(material.index().unwrap_or(0), loaded_mat);
        }
        let mut meshes = Vec::new();
        for mesh in data.0.meshes() {
            let mut primitives = Vec::new();
            for (index, primitive) in mesh.primitives().enumerate() {
                let reader = primitive.reader(|buffer| Some(&data.1[buffer.index()]));

                let mut own_mesh = GltfMesh::new(
                    primitive.mode(),
                    mesh.name()
                        .unwrap_or(format!("Object {}", index).as_str())
                        .to_string(),
                );
                if let Some(position_vertex_attribs) = reader
                    .read_positions()
                    .map(|v| VertexAttribValues::Float32x3(v.collect()))
                {
                    own_mesh.add_vertex_attribute(
                        VertexAttributeType::Position,
                        position_vertex_attribs,
                    );
                }
                if let Some(normal_vertex_attribs) = reader
                    .read_normals()
                    .map(|v| VertexAttribValues::Float32x3(v.collect()))
                {
                    own_mesh
                        .add_vertex_attribute(VertexAttributeType::Normal, normal_vertex_attribs);
                }
                if let Some(tangent_vertex_attribs) = reader
                    .read_tangents()
                    .map(|v| VertexAttribValues::Float32x4(v.collect()))
                {
                    own_mesh
                        .add_vertex_attribute(VertexAttributeType::Tangent, tangent_vertex_attribs);
                }

                if let Some(tex_coords_attribs) = reader
                    .read_tex_coords(0)
                    .map(|v| VertexAttribValues::Float32x2(v.into_f32().collect()))
                {
                    own_mesh.add_vertex_attribute(
                        VertexAttributeType::TextureCoords,
                        tex_coords_attribs,
                    );
                }
                if let Some(indices) = reader.read_indices().map(|v| v.into_u32().collect()) {
                    own_mesh.add_indices(indices);
                }
                primitives.push(GltfPrimitive {
                    mesh: own_mesh,
                    material: primitive.material().index(),
                });
            }
            meshes.push(GltfModel { primitives });
        }
        Ok(HorizonModel {
            meshes,
            materials,
            textures: loaded_textures,
            name: None,
        })
    }

    fn load_gltf_material(
        material: &gltf::material::Material,
        buffer_data: &[gltf::buffer::Data],
        loaded_textures: &mut HashMap<usize, DynamicImage>,
    ) -> Result<GltfMaterial, ImageLoadError> {
        let pbr = material.pbr_metallic_roughness();
        let color = pbr.base_color_factor();
        let base_color_texture = if let Some(ref tex_info) = pbr.base_color_texture() {
            let res = Self::load_texture(buffer_data, &tex_info.texture(), loaded_textures)?;
            Some(res)
        } else {
            None
        };
        let normal_map_texture = if let Some(ref normal_tex) = material.normal_texture() {
            let res = Self::load_texture(buffer_data, &normal_tex.texture(), loaded_textures)?;
            Some(res)
        } else {
            None
        };

        let metallic_roughness_texture = if let Some(metallic_roughness) =
            pbr.metallic_roughness_texture()
        {
            let res =
                Self::load_texture(buffer_data, &metallic_roughness.texture(), loaded_textures)?;
            Some(res)
        } else {
            None
        };
        let occlusion_texture = if let Some(occulsion_texture) = material.occlusion_texture() {
            let res =
                Self::load_texture(buffer_data, &occulsion_texture.texture(), loaded_textures)?;
            Some(res)
        } else {
            None
        };
        let emissive = material.emissive_factor();
        let emissive_texture = if let Some(emissive_info) = material.emissive_texture() {
            let res = Self::load_texture(buffer_data, &emissive_info.texture(), loaded_textures)?;
            Some(res)
        } else {
            None
        };
        let name = if let Some(mat_name) = material.name() {
            mat_name.to_string()
        } else {
            "unnamed".to_string()
        };
        Ok(GltfMaterial {
            base_color_texture,
            emissive_texture,
            normal_map_texture,
            occlusion_texture,
            roughness_texture: metallic_roughness_texture,
            base_color: color,
            double_sided: material.double_sided(),
            pbr_roughness: pbr.roughness_factor(),
            unlit: material.unlit(),
            emissive_color: emissive,
            metallic_factor: pbr.metallic_factor(),
            alpha_mode: material.alpha_mode(),
            name,
        })
    }

    fn load_texture(
        buffer_data: &[gltf::buffer::Data],
        texture: &gltf::Texture,
        loaded_textures: &mut HashMap<usize, DynamicImage>,
    ) -> Result<usize, ImageLoadError> {
        let index = texture.index();
        if let std::collections::hash_map::Entry::Vacant(e) = loaded_textures.entry(index) {
            let img = Texture::create_image_from_gltf_texture(buffer_data, texture)?;
            e.insert(img);
            Ok(index)
        } else {
            Ok(index)
        }
    }
}
#[derive(Clone, Debug)]
pub enum GltfLoadError {
    InnerError(String),
}
