pub mod scriptevent;
pub mod scriptingengine;
pub mod scriptingfunctions;
pub mod util;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[derive(Clone, Debug)]
pub enum ScriptingError {
    MissingComponent(&'static str),
    ModelLoadFailed(String),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone)]
pub enum ResourceType {
    Camera,
    DirectionalLight,
    CamController,
}
