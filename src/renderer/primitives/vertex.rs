use bytemuck::{Pod, Zeroable};
use crate::renderer::primitives::mesh::GltfMesh;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MeshVertexData {
    pub position: [f32; 3],
    pub normals: [f32; 3],
    pub tangent: [f32; 4],
    pub tex_coords: [f32; 2],
    pub vertex_color:u32,
    pub joint_weight:[f32;4],
    pub joint_index:u32,
}

impl Vertex for MeshVertexData {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertexData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute
                {
                    offset:(mem::size_of::<[f32;12]>() + std::mem::size_of::<u32>()) as wgpu::BufferAddress,
                    shader_location:5,
                    format:wgpu::VertexFormat::Float32x4
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32;16]>() + std::mem::size_of::<u32>()) as wgpu::BufferAddress,
                    shader_location:6,
                    format: wgpu::VertexFormat::Uint32
                }
            ],
        }
    }
}
