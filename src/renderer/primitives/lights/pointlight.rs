use bytemuck::*;
use rapier3d::na::{Point3, Vector3};
use specs::*;
#[derive(Component)]
#[storage(VecStorage)]
pub struct PointLight {
    position: Point3<f32>,
    color: wgpu::Color,
    constant: f32,
    linear: f32,
    quadratic: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PointLightRaw {
    pos: [f32; 4],
    color: [f32; 4],
    attenuation_values: [f32; 4],
}

impl PointLight {
    pub fn new(
        position: Point3<f32>,
        color: wgpu::Color,
        constant: f32,
        linear: f32,
        quadratic: f32,
    ) -> Self {
        Self {
            position,
            color,
            constant,
            linear,
            quadratic,
        }
    }
    pub fn to_raw(&self) -> PointLightRaw {
        PointLightRaw {
            attenuation_values: [self.constant, self.linear, self.quadratic, 1.0],
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                self.color.a as f32,
            ],
            pos: [self.position.x, self.position.y, self.position.z, 1.0],
        }
    }
}
