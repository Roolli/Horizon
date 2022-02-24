use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Debug)]
pub enum ComponentTypes {
    AssetIdentifier,
    PhysicsHandle,
    Transform,
    PointLight,
}
