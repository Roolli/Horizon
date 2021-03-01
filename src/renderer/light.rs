use __core::ops::Range;
use bytemuck::*;
use glm::look_at_rh;
use wgpu::BindGroup;

use super::{
    model::HorizonModel,
    primitives::{instance, mesh::Mesh, uniforms::Globals},
};
use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct LightHandle {
    pub index: usize,
}

pub struct Light {
    pub pos: glm::Vec3,
    color: wgpu::Color,
    fov: f32,
    pub depth: Range<f32>,
    pub target_view: wgpu::TextureView,
}

impl Light {
    pub fn new(
        pos: glm::Vec3,
        color: wgpu::Color,
        fov: f32,
        depth: Range<f32>,
        target_view: wgpu::TextureView,
    ) -> Self {
        Self {
            pos,
            color,
            fov,
            depth,
            target_view,
        }
    }

    pub fn to_raw(&self) -> LightRaw {
        let view = glm::look_at_rh(&self.pos, &glm::vec3(0.0, 0.0, 0.0), &glm::Vec3::y());
        let projection = glm::perspective(
            1.0,
            f32::to_radians(self.fov),
            self.depth.start,
            self.depth.end,
        );
        let view_proj =
            glm::Mat4::from(super::state::State::OPENGL_TO_WGPU_MATRIX) * projection * view;
        LightRaw {
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                1.0,
            ],
            position: [self.pos.x, self.pos.y, self.pos.z, 1.0],
            projection: *view_proj.as_ref(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct LightRaw {
    pub projection: [[f32; 4]; 4],
    pub position: [f32; 4],
    pub color: [f32; 4],
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
