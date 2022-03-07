use egui::CursorIcon::Default;
use gltf::mesh::Mode;
use std::collections::BTreeMap;
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
pub enum VertexAttributeType {
    Position,
    Normal,
    Tangent,
    TextureCoords,
}
pub struct GltfMesh {
    mode: wgpu::PrimitiveTopology,
    vertex_attribs: BTreeMap<VertexAttributeType, Vec<f32>>,
    indices: Vec<u32>,
}

impl GltfMesh {
    pub fn new(mode: gltf::mesh::Mode) -> Self {
        let topology = match mode {
            Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
            Mode::Points => wgpu::PrimitiveTopology::PointList,
            _ => wgpu::PrimitiveTopology::TriangleStrip,
        };
        GltfMesh {
            mode: topology,
            vertex_attribs: Default::default(),
            indices: Default::default(),
        }
    }
    pub fn add_indices(&mut self, data: &[u32]) {
        self.indices.copy_from_slice(data);
    }
    pub fn add_vertex_attribute(&mut self, attrib_type: VertexAttributeType, data: &[f32]) {
        self.vertex_attribs[attrib_type] = data;
    }
}
