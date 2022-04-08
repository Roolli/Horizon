use rapier3d::prelude::ColliderHandle;
use specs::*;

#[derive(Component)]
#[storage(VecStorage)]
pub struct CollisionShape {
    pub collider: ColliderHandle,
}
