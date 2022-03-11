use std::collections::HashMap;
use std::ops::Range;

use super::primitives::{material::Material};
use super::primitives::{mesh::Mesh};
use specs::{Component, VecStorage};
use crate::components::gltfmodel::GltfModel;
use crate::renderer::primitives::material::GltfMaterial;

#[derive(Component,Debug)]
#[storage(VecStorage)]
pub struct HorizonModel {
   pub meshes: Vec<GltfModel>,
   pub materials: HashMap<usize,GltfMaterial>,
}
