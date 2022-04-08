use crate::components::physicshandle::PhysicsValues;
use crate::scripting::util::componentconversions::{
    CollisionShapeComponent, PointLightComponent, TransformComponent,
};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentTypes {
    AssetIdentifier,
    PhysicsHandle,
    Transform,
    PointLight,
    CollisionShape,
    None,
}
impl From<u32> for ComponentTypes {
    fn from(val: u32) -> Self {
        match val {
            0 => ComponentTypes::AssetIdentifier,
            1 => ComponentTypes::PhysicsHandle,
            2 => ComponentTypes::Transform,
            3 => ComponentTypes::PointLight,
            4 => ComponentTypes::CollisionShape,
            _ => ComponentTypes::None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentData {
    Empty,
    Transform(TransformComponent),
    Physics(PhysicsValues),
    AssetIdentifier(String),
    PointLight(PointLightComponent),
    CollisionShape(CollisionShapeComponent),
}
