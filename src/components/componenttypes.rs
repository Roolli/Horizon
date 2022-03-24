use crate::components::physicshandle::PhysicsValues;
use crate::scripting::util::componentconversions::{PointLightComponent, TransformComponent};
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
}
#[derive(Debug,Serialize,Deserialize)]
pub enum ComponentData {
    Empty,
    Transform(TransformComponent),
    Physics(PhysicsValues),
    AssetIdentifier(String),
    PointLight(PointLightComponent),
}
