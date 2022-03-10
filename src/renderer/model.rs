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
   pub materials: Vec<GltfMaterial>,
}

// pub trait DrawModel<'a, 'b>
// where
//     'b: 'a,
// {
//     fn draw_mesh(
//         &mut self,
//         mesh: &'b Mesh,
//         material: &'b Material,
//         bind_group: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     );
//     fn draw_mesh_instanced(
//         &mut self,
//         mesh: &'b Mesh,
//         instances: Range<u32>,
//         material: &'b Material,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     );
//     fn draw_model(
//         &mut self,
//         model: &'b HorizonModel,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     );
//     fn draw_model_instanced(
//         &mut self,
//         model: &'b HorizonModel,
//         instances: Range<u32>,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     );
// }
//
// impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
// where
//     'b: 'a,
// {
//     fn draw_mesh(
//         &mut self,
//         mesh: &'b Mesh,
//         material: &'b Material,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     ) {
//         self.draw_mesh_instanced(mesh, 0..1, material, uniforms, light)
//     }
//
//     fn draw_mesh_instanced(
//         &mut self,
//         mesh: &'b Mesh,
//         instances: Range<u32>,
//         material: &'b Material,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     ) {
//         self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//
//         self.set_bind_group(0, &material.bind_group, &[]);
//
//         self.set_bind_group(1, &uniforms, &[]);
//         self.set_bind_group(2, &light, &[]);
//         self.draw_indexed(0..mesh.element_count, 0, instances);
//     }
//
//     fn draw_model(
//         &mut self,
//         model: &'b HorizonModel,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     ) {
//         self.draw_model_instanced(model, 0..1, uniforms, light);
//     }
//
//     fn draw_model_instanced(
//         &mut self,
//         model: &'b HorizonModel,
//         instances: Range<u32>,
//         uniforms: &'b wgpu::BindGroup,
//         light: &'b wgpu::BindGroup,
//     ) {
//         for mesh in &model.meshes {
//             self.draw_mesh_instanced(
//                 mesh,
//                 instances.clone(),
//                 &model.materials[mesh.material],
//                 uniforms,
//                 light,
//             );
//         }
//     }
// }
