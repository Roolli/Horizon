use bytemuck::{Pod, Zeroable};

use crate::renderer::cam::Camera;
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Uniforms {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}
impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
            view_position: [0.0, 0.0, 0.0, 0.0],
        }
    }
    pub fn update_view_proj_matrix(&mut self, cam: &Camera) {
        self.view_position = cam.eye.to_homogeneous().into();
        self.view_proj = cam.build_projection_matrix().into();
    }
}
