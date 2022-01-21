use __core::ops::Range;
use bytemuck::*;

use wgpu::BindGroup;

use crate::{
    renderer::{model::HorizonModel, primitives::mesh::Mesh, state::State},
    resources::camera::Camera,
};



#[derive(Default)]
pub struct DirectionalLight {
    pub direction: glm::Vec3,
    color: wgpu::Color,
}

impl DirectionalLight {
    pub fn new(direction: glm::Vec3, color: wgpu::Color) -> Self {
        Self { direction, color }
    }

    pub fn to_raw(&self, cam: &Camera) -> DirectionalLightRaw {
        let view = glm::look_at_rh(&glm::vec3(0.0, 0.0, 0.0), &self.direction, &glm::Vec3::y());
        let proj = Self::get_frustum_bounding_box(&self, cam, cam.z_near, cam.z_far, &view);
        let view_proj =
            glm::Mat4::identity() * glm::Mat4::from(State::OPENGL_TO_WGPU_MATRIX) * proj * view;
        DirectionalLightRaw {
            direction: [self.direction.x, self.direction.y, self.direction.z, 1.0],
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                1.0,
            ],
            projection: view_proj.into(),
        }
    }
    fn get_frustum_bounding_box(
        &self,
        cam: &Camera,
        znear: f32,
        zfar: f32,
        light_space: &glm::Mat4,
    ) -> glm::Mat4 {
        let cam_view = glm::look_at_rh(&cam.eye, &cam.target, &cam.up);
        let cam_inverse = cam_view.try_inverse().unwrap();

        let tan_half_horizontal_fov = f32::tan(f32::to_radians(cam.fov_y / 2.0));
        let tan_half_vertical_fov = f32::tan(f32::to_radians((cam.fov_y * cam.aspect_ratio) / 2.0));
        let x_near = znear * tan_half_horizontal_fov;
        let x_far = zfar * tan_half_horizontal_fov;

        let y_near = znear * tan_half_vertical_fov;
        let y_far = zfar * tan_half_vertical_fov;

        let frustum_corners = [
            // near face
            glm::Vec4::new(x_near, y_near, znear, 1.0),
            glm::Vec4::new(-x_near, y_near, znear, 1.0),
            glm::Vec4::new(x_near, -y_near, znear, 1.0),
            glm::Vec4::new(-x_near, -y_near, znear, 1.0),
            // far face
            glm::Vec4::new(x_far, y_far, zfar, 1.0),
            glm::Vec4::new(-x_far, y_far, zfar, 1.0),
            glm::Vec4::new(x_far, -y_far, zfar, 1.0),
            glm::Vec4::new(-x_far, -y_far, zfar, 1.0),
        ];

        let mut frustum_corners_l: Vec<glm::Vec4> = Vec::new();

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;
        for i in 0..8 {
            // world space
            let vw = cam_inverse * frustum_corners[i];
            frustum_corners_l.push(light_space * vw);
            min_x = f32::min(min_x, frustum_corners_l[i].x);
            min_y = f32::min(min_y, frustum_corners_l[i].y);
            min_z = f32::min(min_z, frustum_corners_l[i].z);

            max_x = f32::max(max_x, frustum_corners_l[i].x);
            max_y = f32::max(max_y, frustum_corners_l[i].y);
            max_z = f32::max(max_z, frustum_corners_l[i].z);
        }

        glm::ortho_rh(min_x, max_x, min_y, max_y, min_z, max_z)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct DirectionalLightRaw {
    pub projection: [[f32; 4]; 4],
    pub direction: [f32; 4],
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
