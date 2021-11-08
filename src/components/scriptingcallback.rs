use specs::*;

use crate::scripting::lifecycleevents::LifeCycleEvent;
#[cfg(target_arch = "wasm32")]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: js_sys::Function,
}

#[cfg(target_arch = "wasm32")]
impl ScriptingCallback {
    pub fn new(callback: js_sys::Function) -> Self {
        Self { callback }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: rusty_v8::Global<rusty_v8::Function>,
}
