use bytemuck::{Pod, Zeroable};
use rapier3d::na::{Matrix4, Quaternion, Translation3, UnitQuaternion, Vector3, Vector4};

use specs::{Component, Entity, VecStorage};

//TODO: move to components
#[derive(Component)]
#[storage(VecStorage)]
pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    /// The model of the instance
    pub model: Entity,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}
impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Instance {
    pub fn new(pos: Vector3<f32>, rot: UnitQuaternion<f32>, scale: Vector3<f32>, entity: Entity) -> Self {
        Self {
            position: pos,
            rotation: rot,
            scale,
            model: entity,
        }
    }
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model:(Matrix4::new_translation(&self.position) * self.rotation.to_rotation_matrix().to_homogeneous()).append_nonuniform_scaling(&self.scale)
            .into(),
        }
    }
}
