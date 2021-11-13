use specs::Component;
use specs::DenseVecStorage;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(PartialEq, Eq, Hash, Component)]
pub enum ScriptEvent {
    Init = 0,
    OnResourceStart = 1,
    OnTick = 2,
    OnWindowEvent = 4,
    OnResourceStop = 8,
    BeforeRender = 16,
    AfterRender = 32,
    Cleanup = 64,
    OnMouseMove = 128,
    OnKeyDown = 256,
}
