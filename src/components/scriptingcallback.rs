use deno_core::v8;
use deno_core::v8::Handle;
use specs::*;

// #[cfg(not(target_arch = "wasm32"))]
// use crate::scripting::scriptingengine::ScriptingEngineState;
// #[cfg(not(target_arch = "wasm32"))]
// use crate::V8ScriptingEngine;
use crate::HorizonScriptingEngine;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
    pub fn get_callback(&self) -> &js_sys::Function {
        &self.callback
    }
}
#[cfg(target_arch = "wasm32")]
impl ExecuteFunction for ScriptingCallback {
    fn execute_with_no_args(&self) {
        self.get_callback().call0(&JsValue::UNDEFINED).unwrap();
    }

    fn execute_with_args(&self, args: CallbackArgs) {
        self.get_callback()
            .call1(&JsValue::NULL, &JsValue::from(args[0]))
            .unwrap();
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: deno_core::v8::Global<deno_core::v8::Function>,
}
#[cfg(not(target_arch = "wasm32"))]
impl<'s> ExecuteFunction<'s> for ScriptingCallback {
    type InvokeParameters = (&'s mut HorizonScriptingEngine);
    fn execute_with_no_args(&self, additional_args: Self::InvokeParameters) {
        let engine = additional_args;
        let js = &mut engine.js_runtime;
        let scope = &mut js.handle_scope();
        let recv = v8::Integer::new(scope, 1).into();
        self.callback.open(scope).call(scope, recv, &[]);
    }

    fn execute_with_args(&self, additional_args: Self::InvokeParameters, args: CallbackArgs) {
        let engine = additional_args;
        let js = &mut engine.js_runtime;
        let scope = &mut js.handle_scope();
        let recv = v8::Integer::new(scope, 1).into();
        match args {
            CallbackArgs::Tick(t) => {
                let val = v8::Number::new(scope, t as f64).into();
                self.callback.open(scope).call(scope, recv, &[val]);
            }
            CallbackArgs::KeyboardEvent(keycode) => {
                let val = v8::Integer::new(scope, keycode as i32).into();
                self.callback.open(scope).call(scope, recv, &[val]);
            }
            CallbackArgs::MouseMoveEvent((rel_x, rel_y)) => {
                let x = v8::Number::new(scope, rel_x).into();
                let y = v8::Number::new(scope, rel_y).into();
                self.callback.open(scope).call(scope, recv, &[x, y]);
            }
            CallbackArgs::MouseClickEvent(button_id) => {
                let num = v8::Integer::new(scope, button_id as i32).into();
                self.callback.open(scope).call(scope, recv, &[num]);
            }
            CallbackArgs::None => {
                self.callback.open(scope).call(scope, recv, &[]);
            }
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl ScriptingCallback {
    pub fn new(callback: deno_core::v8::Global<deno_core::v8::Function>) -> Self {
        Self { callback }
    }
    pub fn get_callback(&self) -> &deno_core::v8::Global<deno_core::v8::Function> {
        &self.callback
    }
}

pub trait ExecuteFunction<'s> {
    type InvokeParameters;
    fn execute_with_no_args(&self, additional_args: Self::InvokeParameters);
    fn execute_with_args(&self, additional_args: Self::InvokeParameters, args: CallbackArgs);
}
#[derive(Debug)]
pub enum CallbackArgs {
    None,
    Tick(f32),
    KeyboardEvent(u32),
    MouseClickEvent(u16),
    MouseMoveEvent((f64, f64)),
}
