use bytemuck::{Pod, Zeroable};
use rapier3d::na::Matrix4;

use crate::resources::camera::Camera;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Globals {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    num_lights: [u32; 4],
}
impl Globals {
    pub fn new(point_light_count: u32, spot_light_count: u32) -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            view_position: [0.0, 0.0, 0.0, 0.0],
            num_lights: [point_light_count, spot_light_count, 0, 0],
        }
    }
    pub fn update_view_proj_matrix(&mut self, cam: &Camera) {
        self.view_position = cam.eye.to_homogeneous().into();
        self.view_proj = cam.build_view_projection_matrix().into();
    }
    pub fn set_point_light_count(&mut self, new_count: u32) {
        self.num_lights[0] = new_count;
    }
    pub fn set_spot_light_count(&mut self, new_count: u32) {
        self.num_lights[1] = new_count;
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ShadowUniforms {
    proj: [[f32; 4]; 4],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CanvasConstants {
    pub size: [f32; 2],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TileInfo {
    pub tile_size: i32,
    pub tile_count_x: i32,
    pub tile_count_y: i32,
    pub num_tiles: u32,
    pub num_tile_light_slot: u32,
}
