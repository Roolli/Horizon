use rapier3d::math::Point;
use rapier3d::na::{Matrix4, Point3, Vector3};
use specs::*;
#[derive(Component)]
#[storage(VecStorage)]
pub struct SpotLight {
    position: Point3<f32>,
    direction: Matrix4<f32>,
    color: Vector3<f32>,
    radius: f32,
    /// Requires cos
    inner_cutoff: f32,
    /// Requires cos
    outer_cutoff: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotLightRaw {
    pos: [f32; 4],
    dir: [[f32; 4]; 4],
    color: [f32; 3],
    radius: f32,
    cutoffs: [f32; 4],
}
impl SpotLight {
    pub fn new(
        position: Point3<f32>,
        direction: Matrix4<f32>,
        color: Vector3<f32>,
        radius: f32,
        inner_cutoff: f32,
        outer_cutoff: f32,
    ) -> Self {
        Self {
            position,
            color,
            direction,
            radius,
            inner_cutoff,
            outer_cutoff,
        }
    }
    pub fn to_raw(&self) -> SpotLightRaw {
        SpotLightRaw {
            radius: self.radius,
            color: [self.color.x, self.color.y, self.color.z],
            pos: [self.position.x, self.position.y, self.position.z, 1.0],
            dir: self.direction.into(),
            cutoffs: [self.inner_cutoff, self.outer_cutoff, 1.0, 1.0],
        }
    }
}
