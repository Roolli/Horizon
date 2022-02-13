use serde::Deserialize;
use serde::Serialize;

use crate::scripting::util::glmconversion::*;
use crate::scripting::util::RigidBodyType;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityInfo {
    pub components: Vec<Component>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub component_type: String,
    pub scale: Option<Vec3>,
    pub position: Option<Vec3>,
    pub rotation: Option<Vec4>,
    pub model: Option<u32>,
    pub attenuation: Option<Vec3>,
    pub color: Option<Vec4>,
    pub body_type: Option<RigidBodyType>,
    pub mass: Option<f64>,
    pub lock_rotation: Option<LockRotation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockRotation {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}
