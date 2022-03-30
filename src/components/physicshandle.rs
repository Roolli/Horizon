use crate::scripting::util::glmconversion::Vec3;
use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Component, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct PhysicsHandle {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handles: Vec<ColliderHandle>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PhysicsValues {
    pub angular_damping: f32,
    pub linear_damping: f32,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
}
