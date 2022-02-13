use bytemuck::{Pod, Zeroable};
use rapier3d::na::{Matrix4, Unit, UnitQuaternion, Vector3};

use specs::prelude::*;
use specs::{Component, VecStorage};
#[derive(Component, Copy, Clone, Debug)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub model: Option<Entity>,
}


impl Transform {
    pub fn new(
        position: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        scale: Vector3<f32>,
        model: Option<Entity>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            model,
        }
    }
    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }
    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.rotation = rotation;
    }
    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
    }
    pub fn set_model(&mut self, model_id: Option<Entity>) {
        self.model = model_id;
    }
    pub fn get_model(self) -> Option<Entity> {
        self.model
    }
    pub fn get_position(&self) -> Vector3<f32> {
        self.position
    }
    pub fn get_rotation(&self) -> UnitQuaternion<f32> {
        self.rotation
    }
    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale
    }

    pub fn to_raw(&self) -> TransformRaw {
        TransformRaw {
            data: (Matrix4::new_translation(&self.position) *self.rotation.to_rotation_matrix().to_homogeneous()).append_nonuniform_scaling(&self.scale)
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
        let mat4 = Matrix4::from(self.data);
        let inverted = mat4.try_inverse().unwrap();
        let transposed = inverted.transpose();
        transposed.into()
    }
}
