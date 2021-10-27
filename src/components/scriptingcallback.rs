use specs::*;

use crate::scripting::lifecycleevents::LifeCycleEvent;
#[cfg(target_arch = "wasm32")]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: js_sys::Function,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: v8::Global<v8::Function>,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallbackType {
    callback_type: LifeCycleEvent,
}
