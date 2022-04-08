use serde::Deserialize;
use serde::Serialize;

use crate::scripting::util::glmconversion::*;
use crate::scripting::util::RigidBodyType;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityInfo {
    pub transform: Component,
    pub components: Vec<Component>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub component_type: String,
    pub scale: Option<Vec3>,
    pub position: Option<Vec3>,
    pub rotation: Option<Vec3>,
    pub model: Option<u32>,
    pub radius: Option<f32>,
    pub color: Option<Vec3>,
    pub body_type: Option<RigidBodyType>,
    pub mass: Option<f64>,
    pub collision_shape: Option<ColliderInfo>,
    pub lock_rotation: Option<LockRotation>,
    pub damping: Option<Vec<Damping>>,
}
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColliderInfo {
    pub collision_shape_type: Option<String>,
    pub half_extents: Option<Vec3>,
    pub half_height: Option<f32>,
    pub capsule_type: Option<String>,
    pub radius: Option<f32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockRotation {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Damping {
    pub damping_type: String,
    pub amount: f32,
}
