use rapier3d::na::{Quaternion, Vector3, Vector4};
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
impl From<Vec3> for Vector3<f32> {
    fn from(val: Vec3) -> Self {
        Vector3::new(val.x, val.y, val.z)
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Get a reference to the vec4's z.
    pub fn z(&self) -> f32 {
        self.z
    }

    /// Get a reference to the vec4's x.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Get a reference to the vec4's y.
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Get a reference to the vec4's w.
    pub fn w(&self) -> f32 {
        self.w
    }
}
impl From<Vec4> for Vector4<f32> {
    fn from(val: Vec4) -> Self {
        Vector4::new(val.x, val.y, val.z, val.w)
    }
}
impl From<Vec4> for Quaternion<f32> {
    fn from(val: Vec4) -> Self {
        rapier3d::prelude::nalgebra::Quaternion::from_vector(val.into())
    }
}
