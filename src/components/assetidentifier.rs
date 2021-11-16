use specs::*;

#[derive(Component)]
#[storage(VecStorage)]
pub struct AssetIdentifier(pub String);
