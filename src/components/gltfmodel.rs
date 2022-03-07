use crate::renderer::primitives::material::GltfMaterial;
use crate::renderer::primitives::mesh::GltfMesh;

pub struct GltfModel {
    pub primitives: Vec<GltfPrimitive>,
}

pub struct GltfPrimitive {
    pub material: Option<GltfMaterial>,
    pub mesh: GltfMesh,
}
