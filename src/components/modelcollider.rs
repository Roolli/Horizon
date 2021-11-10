use rapier3d::geometry::ColliderBuilder;
use specs::*;

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct ModelCollider(pub ColliderBuilder);
