#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::components::physicshandle::PhysicsValues;
use crate::scripting::util::componentconversions::{PointLightComponent, TransformComponent};

#[cfg_attr(target_arch="wasm32",wasm_bindgen)]
#[derive(Debug)]
pub enum ComponentTypes {
    AssetIdentifier,
    PhysicsHandle,
    Transform,
    PointLight,
}
//#[derive(Debug)]
pub enum ComponentData
{
    Empty,
    Transform(TransformComponent),
    Physics(PhysicsValues),
    AssetIdentifier(String),
    PointLight(PointLightComponent)
}