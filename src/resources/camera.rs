use rapier3d::na::{Matrix4, Point3, Vector3};
use crate::renderer::state::State;

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}
impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Matrix4::new_perspective(
            self.aspect_ratio,
            f32::to_radians(self.fov_y),
            self.z_near,
            self.z_far,
        );
        let mut reversed_z_matrix = Matrix4::identity();
        *reversed_z_matrix.get_mut(10).unwrap() = -1.0;
        *reversed_z_matrix.get_mut(14).unwrap() = 1.0;

        Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * (reversed_z_matrix * proj * view)
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
       Matrix4::look_at_rh(&self.eye, &self.target, &self.up)
    }
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(
            self.aspect_ratio,
            f32::to_radians(self.fov_y),
            self.z_near,
            self.z_far,
        )
    }
    pub fn look_at(&mut self, point: Point3<f32>) {
        self.target = point;
    }
    pub fn set_position(&mut self, point: Point3<f32>) {
        self.eye = point;
    }
}
