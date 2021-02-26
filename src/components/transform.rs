use bytemuck::{Pod, Zeroable};
use specs::{storage, Component, VecStorage};
#[derive(Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
}

impl Transform {
    pub fn new(position: glm::Vec3, rotation: glm::Quat, scale: glm::Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
    pub fn to_raw(&self) -> TransformRaw {
        TransformRaw {
            data: (glm::translate(&glm::Mat4::identity(), &self.position)
                * glm::quat_to_mat4(&self.rotation)
                * glm::scale(&glm::Mat4::identity(), &self.scale))
            .into(),
        }
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TransformRaw {
    data: [[f32; 4]; 4],
}
impl TransformRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TransformRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}
