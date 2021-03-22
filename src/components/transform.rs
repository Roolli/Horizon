use bytemuck::{Pod, Zeroable};

use specs::prelude::*;
use specs::{Component, VecStorage};
#[derive(Component, Copy, Clone)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
    pub model: Entity,
}

impl Transform {
    pub fn new(position: glm::Vec3, rotation: glm::Quat, scale: glm::Vec3, model: Entity) -> Self {
        Self {
            position,
            rotation,
            scale,
            model,
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
    pub fn get_normal_matrix(&self) -> [[f32; 4]; 4] {
        let mat4 = glm::Mat4::from(self.data);
        let inverted = mat4.try_inverse().unwrap();
        let transposed = inverted.transpose();
        transposed.into()
    }
}
