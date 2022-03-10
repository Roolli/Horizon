use crate::renderer::primitives::material::GltfMaterial;
use crate::renderer::primitives::mesh::GltfMesh;
use specs::*;

#[derive(Debug)]
pub struct GltfModel {
    pub primitives: Vec<GltfPrimitive>,
}
#[derive(Debug)]
pub struct GltfPrimitive {
    pub material: Option<usize>,
    pub mesh: GltfMesh,
}
#[derive(Component,Debug)]
#[storage(VecStorage)]
pub struct RawMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub material_index: usize,
    pub name:String,
}
#[derive(Component)]
#[storage(VecStorage)]
pub struct RawMaterial
{
    pub bind_group:wgpu::BindGroup,
}
pub struct RawModel {
    pub meshes: Vec<RawMesh>,
    pub materials: Vec<RawMaterial>,
}