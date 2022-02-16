use __core::ops::Range;
use bytemuck::*;
use rapier3d::na::{Matrix4, Point3, Vector3, Vector4};

use wgpu::BindGroup;

use crate::{Projection, renderer::{model::HorizonModel, primitives::mesh::Mesh, state::State}, resources::camera::Camera};



pub struct DirectionalLight {
    pub direction: Point3<f32>,
    color: wgpu::Color,
}

impl DirectionalLight {
    pub fn new(direction: Point3<f32>, color: wgpu::Color) -> Self {
        Self { direction, color }
    }

    pub fn to_raw(&self, cam: &Camera,proj:&Projection) -> DirectionalLightRaw {
        let view = Matrix4::look_at_rh(&Point3::origin(),&self.direction,&Vector3::y());
        let proj = Self::get_frustum_bounding_box(self, cam, proj,  &view);
        let view_proj =
           Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * proj * view;
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
        proj:&Projection,
        light_space: &Matrix4<f32>,
    ) -> Matrix4<f32> {
        let cam_view = cam.get_view_matrix();

        let cam_inverse = cam_view.try_inverse().unwrap();

        let tan_half_horizontal_fov = f32::tan(f32::to_radians(proj.fov_y / 2.0));
        let tan_half_vertical_fov = f32::tan(f32::to_radians((proj.fov_y * proj.aspect_ratio) / 2.0));
        let x_near = proj.z_near * tan_half_horizontal_fov;
        let x_far = proj.z_far * tan_half_horizontal_fov;

        let y_near = proj.z_near * tan_half_vertical_fov;
        let y_far = proj.z_far * tan_half_vertical_fov;

        let frustum_corners = [
            // near face
            Vector4::new(x_near, y_near, proj.z_near, 1.0),
            Vector4::new(-x_near, y_near, proj.z_near, 1.0),
            Vector4::new(x_near, -y_near, proj.z_near, 1.0),
            Vector4::new(-x_near, -y_near, proj.z_near, 1.0),
            // far face
            Vector4::new(x_far, y_far, proj.z_far, 1.0),
            Vector4::new(-x_far, y_far, proj.z_far, 1.0),
            Vector4::new(x_far, -y_far, proj.z_far, 1.0),
            Vector4::new(-x_far, -y_far, proj.z_far, 1.0),
        ];

        let mut frustum_corners_l: Vec<Vector4<f32>> = Vec::new();

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

        Matrix4::new_orthographic(min_x, max_x, min_y, max_y, min_z, max_z)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct DirectionalLightRaw {
    pub projection: [[f32; 4]; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
}
// TODO: add proper debug lights
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
