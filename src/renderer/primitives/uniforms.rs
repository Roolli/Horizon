use bytemuck::{Pod, Zeroable};
use rapier3d::na::Matrix4;

use crate::resources::camera::Camera;
use crate::resources::projection::Projection;
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
    pub fn update_view_proj_matrix(&mut self, cam: &Camera, proj: &Projection) {
        self.view_position = cam.position.to_homogeneous().into();
        self.view_proj = (proj.calc_proj_matrix() * cam.get_view_matrix()).into();
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
impl Default for TileInfo {
    fn default() -> Self {
        Self {
            tile_size: 16,
            num_tile_light_slot: 128,
            tile_count_y: 0,
            tile_count_x: 0,
            num_tiles: 0,
        }
    }
}
impl TileInfo {
    pub fn calculate_light_id_buffer_size(
        &mut self,
        width: f32,
        height: f32,
    ) -> wgpu::BufferAddress {
        // return Math.floor( (t + f - 1) / f);
        self.tile_count_x =
            f32::floor((width + (self.tile_size - 1) as f32) / self.tile_size as f32) as i32;
        self.tile_count_y =
            f32::floor((height + (self.tile_size - 1) as f32) / self.tile_size as f32) as i32;
        self.num_tiles = (self.tile_count_x * self.tile_count_y) as u32;
        (std::mem::size_of::<u32>()
            * (self.num_tile_light_slot + 1) as usize
            * self.num_tiles as usize) as wgpu::BufferAddress
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SkyboxUniform {
    pub projection_inverse: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightCullingUniforms {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}
impl LightCullingUniforms {
    pub fn new(projection: &Projection, view: &Camera) -> Self {
        LightCullingUniforms {
            projection: projection.calc_proj_matrix().data.0,
            view: view.get_view_matrix().data.0,
        }
    }
}
