use __core::ops::Range;
use bytemuck::*;
use wgpu::BindGroup;

use super::{
    model::HorizonModel,
    primitives::{instance, mesh::Mesh, uniforms::Uniforms},
};
#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct Light {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
}
pub trait DrawLight<'a, 'b>
where
    'b: 'a,
{
    fn draw_light_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b BindGroup, light: &'b BindGroup);
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        uniforms: &'b BindGroup,
        light: &'b BindGroup,
    );
    fn draw_light_model(
        &mut self,
        model: &'b HorizonModel,
        uniforms: &'b BindGroup,
        light: &'b BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'b HorizonModel,
        instances: Range<u32>,
        uniforms: &'b BindGroup,
        light: &'b BindGroup,
    );
}
impl<'a, 'b> DrawLight<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(&mut self, mesh: &'b Mesh, uniforms: &'b BindGroup, light: &'b BindGroup) {
        self.draw_light_mesh_instanced(mesh, 0..1, uniforms, light);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        uniforms: &'b BindGroup,
        light: &'b BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &uniforms, &[]);
        self.set_bind_group(1, &light, &[]);
        self.draw_indexed(0..mesh.element_count, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b HorizonModel,
        uniforms: &'b BindGroup,
        light: &'b BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, uniforms, light);
    }

    fn draw_light_model_instanced(
        &mut self,
        model: &'b HorizonModel,
        instances: Range<u32>,
        uniforms: &'b BindGroup,
        light: &'b BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(mesh, instances.clone(), uniforms, light);
        }
    }
}
