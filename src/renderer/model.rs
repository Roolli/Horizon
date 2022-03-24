use image::DynamicImage;
use std::collections::HashMap;
use std::ops::Range;

use super::primitives::material::Material;
use super::primitives::mesh::Mesh;
use crate::components::gltfmodel::GltfModel;
use crate::renderer::primitives::material::GltfMaterial;
use specs::{Component, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct HorizonModel {
    pub meshes: Vec<GltfModel>,
    pub materials: HashMap<usize, GltfMaterial>,
    pub textures: HashMap<usize, DynamicImage>,
    pub name: Option<String>,
}
