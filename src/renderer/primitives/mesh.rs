use std::usize;

use rapier3d::prelude::nalgebra::*;

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub element_count: u32,
    pub material: usize,
    pub points: Vec<Point3<f32>>,
}
