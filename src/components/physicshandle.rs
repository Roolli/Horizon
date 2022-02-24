use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};
use specs::{Component, VecStorage};
use serde::{Serialize,Deserialize};

#[derive(Component, Clone, Copy,Serialize,Deserialize)]
#[storage(VecStorage)]
pub struct PhysicsHandle {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle, // Might not be needed as remove doesn't need it.
}
