use super::state::State;

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
    pub fn build_projection_matrix(&self) -> glm::Mat4 {
        let view = glm::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = glm::perspective(
            self.aspect_ratio,
            f32::to_radians(self.fov_y),
            self.z_near,
            self.z_far,
        );
        glm::Mat4::from(State::OPENGL_TO_WGPU_MATRIX) * proj * view
    }
}
