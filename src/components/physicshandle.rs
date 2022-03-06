use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};
use rapier3d::na::Vector3;
use specs::{Component, VecStorage};
use serde::{Serialize,Deserialize};
use crate::scripting::util::glmconversion::Vec3;

#[derive(Component, Clone, Copy,Serialize,Deserialize)]
#[storage(VecStorage)]
pub struct PhysicsHandle {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle, // Might not be needed as remove doesn't need it.
}
#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct PhysicsValues {
    pub angular_damping:f32,
    pub linear_damping:f32,
    pub linear_velocity:Vec3,
    pub angular_velocity:Vec3,
    pub mass:f32,
}