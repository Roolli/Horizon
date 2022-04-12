use bytemuck::*;
use rapier3d::na::{Point3, Vector3};
use specs::*;
#[derive(Component, Copy, Clone)]
#[storage(VecStorage)]
pub struct PointLight {
    pub color: Vector3<f32>,
    pub radius: f32,
    pub attached_to: Option<Entity>,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PointLightRaw {
    pos: [f32; 4],
    color: [f32; 3],
    radius: f32,
}

impl PointLight {
    pub fn new(color: Vector3<f32>, radius: f32, attached_to: Option<Entity>) -> Self {
        Self {
            color,
            radius,
            attached_to,
        }
    }
    pub fn attach_to(&mut self, ent: Entity) {
        self.attached_to = Some(ent);
    }
    pub fn to_raw(&self, pos: Vector3<f32>) -> PointLightRaw {
        PointLightRaw {
            radius: self.radius,
            color: [self.color.x, self.color.y, self.color.z],
            pos: [pos.x, pos.y, pos.z, 1.0],
        }
    }
}
