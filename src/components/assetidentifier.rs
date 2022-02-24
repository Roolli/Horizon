use specs::*;
use serde::{Serialize,Deserialize};

#[derive(Component,Serialize,Deserialize)]
#[storage(VecStorage)]
pub struct AssetIdentifier(pub String);
