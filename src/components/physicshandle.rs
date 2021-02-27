use rapier3d::{dynamics::RigidBodyHandle, geometry::ColliderHandle};
use specs::{Component, VecStorage};

#[derive(Component, Clone, Copy)]
#[storage(VecStorage)]
pub struct PhysicsHandle {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}
