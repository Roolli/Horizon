use std::collections::HashMap;
use std::ops::Range;
use crate::renderer::primitives::material::GltfMaterial;
use crate::renderer::primitives::mesh::GltfMesh;
use specs::*;
use wgpu::BindGroup;
use crate::BindGroupContainer;

#[derive(Debug)]
pub struct GltfModel {
    pub primitives: Vec<GltfPrimitive>,
}
#[derive(Debug)]
pub struct GltfPrimitive {
    pub material: Option<usize>,
    pub mesh: GltfMesh,
}

pub struct RawMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub material_index: usize,
    pub name:String,
    pub index_buffer_len: u32,
}

pub struct RawMaterial
{
    pub bind_group_container:BindGroupContainer,
}
#[derive(Component)]
#[storage(VecStorage)]
pub struct RawModel {
    pub meshes: Vec<RawMesh>,
    pub materials: HashMap<usize,RawMaterial>,
}
pub trait DrawModel<'a, 'b>
    where
        'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh:&'a RawMesh,
        material:&'a RawMaterial,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh:&'a RawMesh,
        material:&'a RawMaterial,
        instances: Range<u32>,
    );
    fn draw_model(
        &mut self,
        model:&'b RawModel,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'b RawModel,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
    where
        'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh:&'a RawMesh,
        material:&'a RawMaterial,

    ) {
        self.draw_mesh_instanced(mesh,material,0..1)
    }
    fn draw_mesh_instanced(
        &mut self,
        mesh:&'a RawMesh,
        material:&'a RawMaterial,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.set_bind_group(1, &material.bind_group_container.bind_group, &[]);
        self.draw_indexed(0..mesh.index_buffer_len, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b RawModel,
    ) {
        self.draw_model_instanced(model, 0..1);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b RawModel,
        instances: Range<u32>,

    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(
                mesh,
                &model.materials[&mesh.material_index],
                instances.clone(),
            );
        }
    }
}
