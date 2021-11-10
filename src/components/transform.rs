use bytemuck::{Pod, Zeroable};

use specs::prelude::*;
use specs::{Component, VecStorage};
#[derive(Component, Copy, Clone)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
    pub model: Option<Entity>,
}
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

impl Transform {
    pub fn new(
        position: glm::Vec3,
        rotation: glm::Quat,
        scale: glm::Vec3,
        model: Option<Entity>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            model,
        }
    }
    pub fn set_position(&mut self, positon: glm::Vec3) {
        self.position = positon;
    }
    pub fn set_rotation(&mut self, rotation: glm::Quat) {
        self.rotation = rotation;
    }
    pub fn set_scale(&mut self, scale: glm::Vec3) {
        self.scale = scale;
    }
    pub fn set_model(&mut self, model_id: Option<Entity>) {
        self.model = model_id;
    }
    pub fn get_model(self) -> Option<Entity> {
        self.model
    }
    pub fn get_position(&self) -> glm::Vec3 {
        self.position
    }
    pub fn get_rotation(&self) -> glm::Quat {
        self.rotation
    }
    pub fn get_scale(&self) -> glm::Vec3 {
        self.scale
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
