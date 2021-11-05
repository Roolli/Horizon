use crate::renderer::state::State;

pub struct Camera {
    pub eye: glm::Vec3,
    pub target: glm::Vec3,
    pub up: glm::Vec3,
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}
impl Camera {
    pub fn build_view_projection_matrix(&self) -> glm::Mat4 {
        let view = glm::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = glm::perspective(
            self.aspect_ratio,
            f32::to_radians(self.fov_y),
            self.z_near,
            self.z_far,
        );
        let mut reversed_z_matrix = glm::Mat4::identity();
        *reversed_z_matrix.get_mut(10).unwrap() = -1.0;
        *reversed_z_matrix.get_mut(14).unwrap() = 1.0;

        glm::Mat4::from(State::OPENGL_TO_WGPU_MATRIX) * (reversed_z_matrix * proj * view)
    }
    pub fn get_view_matrix(&self) -> glm::Mat4 {
        glm::look_at_rh(&self.eye, &self.target, &self.up)
    }
    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        glm::perspective(
            self.aspect_ratio,
            f32::to_radians(self.fov_y),
            self.z_near,
            self.z_far,
        )
    }
    pub fn look_at(&mut self, point: glm::Vec3) {
        self.target = point;
    }
    pub fn set_position(&mut self, point: glm::Vec3) {
        self.eye = point;
    }
}
