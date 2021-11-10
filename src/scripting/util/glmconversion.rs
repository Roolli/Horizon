use crate::renderer::utils::ecscontainer::ECSContainer;
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
impl From<Vec3> for glm::Vec3 {
    fn from(val: Vec3) -> Self {
        glm::Vec3::new(val.x, val.y, val.z)
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
impl From<Vec4> for glm::Vec4 {
    fn from(val: Vec4) -> Self {
        glm::vec4(val.x, val.y, val.z, val.w)
    }
}
impl From<Vec4> for glm::Quat {
    fn from(val: Vec4) -> Self {
        glm::quat_angle_axis(val.w, &glm::vec3(val.x, val.y, val.z))
    }
}
