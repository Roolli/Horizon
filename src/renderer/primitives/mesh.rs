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
#[derive(Ord, PartialOrd, Eq, PartialEq,Debug)]
pub enum VertexAttributeType {
    Position,
    Normal,
    Tangent,
    TextureCoords,
    VertexColor,
    JointWeight,
    JointIndex,
}
#[derive(Debug)]
pub struct GltfMesh {
    mode: wgpu::PrimitiveTopology,
    vertex_attribs: BTreeMap<VertexAttributeType, VertexAttribValues>,
    indices: Option<Vec<u32>>,
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
    pub fn add_indices(&mut self, data:Vec<u32>) {
        self.indices = Some(data);
    }
    pub fn add_vertex_attribute(&mut self, attrib_type: VertexAttributeType, data: VertexAttribValues) {
        self.vertex_attribs.insert(attrib_type,data);
    }
    pub fn attribute(&self,id:VertexAttributeType) -> Option<&VertexAttribValues>
    {
        self.vertex_attribs.get(&id)
    }
}
// https://github.com/bevyengine/bevy/blob/e369a8ad5138af28a7e760fac3f07b278c27ebb4/crates/bevy_render/src/mesh/mesh/mod.rs
#[derive(Clone,Debug)]
pub enum VertexAttribValues {
    Float32(Vec<f32>),
    Sint32(Vec<i32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<[f32; 2]>),
    Sint32x2(Vec<[i32; 2]>),
    Uint32x2(Vec<[u32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Sint32x3(Vec<[i32; 3]>),
    Uint32x3(Vec<[u32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32x4(Vec<[u32; 4]>),
    Sint16x2(Vec<[i16; 2]>),
    Snorm16x2(Vec<[i16; 2]>),
    Uint16x2(Vec<[u16; 2]>),
    Unorm16x2(Vec<[u16; 2]>),
    Sint16x4(Vec<[i16; 4]>),
    Snorm16x4(Vec<[i16; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Unorm16x4(Vec<[u16; 4]>),
    Sint8x2(Vec<[i8; 2]>),
    Snorm8x2(Vec<[i8; 2]>),
    Uint8x2(Vec<[u8; 2]>),
    Unorm8x2(Vec<[u8; 2]>),
    Sint8x4(Vec<[i8; 4]>),
    Snorm8x4(Vec<[i8; 4]>),
    Uint8x4(Vec<[u8; 4]>),
    Unorm8x4(Vec<[u8; 4]>),
}
