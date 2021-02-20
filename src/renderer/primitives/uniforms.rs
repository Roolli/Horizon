use bytemuck::{Pod, Zeroable};

use crate::renderer::cam::Camera;
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Globals {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    num_lights: [u32; 4],
}
impl Globals {
    pub fn new(light_num: u32) -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
            view_position: [0.0, 0.0, 0.0, 0.0],
            num_lights: [light_num, 0, 0, 0],
        }
    }
    pub fn update_view_proj_matrix(&mut self, cam: &Camera) {
        self.view_position = cam.eye.to_homogeneous().into();
        self.view_proj = cam.build_projection_matrix().into();
    }
}
