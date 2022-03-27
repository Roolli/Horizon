use bytemuck::*;
use rapier3d::na::{Point3, Vector3};
use specs::*;
#[derive(Component, Copy, Clone)]
#[storage(VecStorage)]
pub struct PointLight {
    pub position: Point3<f32>,
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
    pub fn new(
        position: Point3<f32>,
        color: Vector3<f32>,
        radius: f32,
        attached_to: Option<Entity>,
    ) -> Self {
        Self {
            position,
            color,
            radius,
            attached_to,
        }
    }
    pub fn attach_to(&mut self, ent: Entity) {
        self.attached_to = Some(ent);
    }
    pub fn to_raw(&self) -> PointLightRaw {
        PointLightRaw {
            radius: self.radius,
            color: [self.color.x, self.color.y, self.color.z],
            pos: [self.position.x, self.position.y, self.position.z, 1.0],
        }
    }
}
