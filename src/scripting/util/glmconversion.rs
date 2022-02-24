use rapier3d::na::{Point3, Quaternion, UnitQuaternion, Vector3, Vector4};
use serde::Deserialize;
use serde::Serialize;
use wgpu::Color;

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
impl From<Vector3<f32>> for Vec3{
    fn from(val: Vector3<f32>) -> Self {
        Vec3::new(val.x,val.y,val.z)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec4<T> where T: Copy + Clone{
    x: T,
    y: T,
    z: T,
    w: T,
}

impl<T> Vec4<T> where T: Copy + Clone  {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }

    /// Get a reference to the vec4's z.
    pub fn z(&self) -> T {
        self.z
    }

    /// Get a reference to the vec4's x.
    pub fn x(&self) -> T {
        self.x
    }

    /// Get a reference to the vec4's y.
    pub fn y(&self) -> T {
        self.y
    }

    /// Get a reference to the vec4's w.
    pub fn w(&self) -> T {
        self.w
    }
}
impl From<Vec4<f32>> for Vector4<f32>  {
    fn from(val: Vec4<f32>) -> Self {
        Vector4::new(val.x, val.y, val.z, val.w)
    }
}

impl From<Vector4<f32>> for Vec4<f32> {
    fn from(val:Vector4<f32>)-> Self
    {
        Vec4::new(val.x,val.y,val.z,val.w)
    }
}
impl From<wgpu::Color> for Vec4<f64> {
    fn from(val: Color) -> Self {
        Vec4::new(val.r,val.g,val.b,val.a)
    }
}

impl From<Vec3> for UnitQuaternion<f32> {
    fn from(val: Vec3) -> Self {
        rapier3d::prelude::nalgebra::UnitQuaternion::from_euler_angles(val.x,val.y,val.z)
    }
}
impl From<UnitQuaternion<f32>> for Vec3{
    fn from(val: UnitQuaternion<f32>) -> Self {
        let (x,y,z) = val.euler_angles();
        Vec3::new(x,y,z)
    }
}
impl From<Point3<f32>> for Vec3 {
    fn from(val: Point3<f32>) -> Self {
        Vec3::new(val.x,val.y,val.z)
    }
}

