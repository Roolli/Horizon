use specs::{storage, Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
}
