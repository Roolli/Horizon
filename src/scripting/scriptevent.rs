use specs::Component;
use specs::DenseVecStorage;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[repr(i32)]
#[derive(PartialEq, Eq, Hash, Component, Debug)]
pub enum ScriptEvent {
    Init = 0,
    OnResourceStart = 1,
    OnTick = 2,
    OnWindowEvent = 3,
    OnResourceStop = 4,
    BeforeRender = 5,
    AfterRender = 6,
    Cleanup = 7,
    OnMouseMove = 8,
    OnKeyDown = 9,
}
impl ScriptEvent {
    pub fn from_number(val: i32) -> Self {
        let enum_val: ScriptEvent = unsafe { std::mem::transmute(val) };
        enum_val
    }
}
