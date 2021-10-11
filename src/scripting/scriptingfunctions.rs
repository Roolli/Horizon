use rusty_v8 as v8;
use std::collections::HashMap;
use std::io::Write;
use std::{convert::TryFrom, io::stdout};
use v8::{Function, Global};

use super::scriptingengine::V8ScriptingEngine;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
pub struct ScriptingFunctions;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ScriptingFunctions {
    pub lifecycle_event_storage: HashMap<LifeCycleEvent, Vec<&js_sys::Function>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ScriptingFunctions {
    pub fn print(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let obj = args.get(0);
        let try_catch_scope = &mut v8::TryCatch::new(scope);
        let string = obj.to_string(try_catch_scope).unwrap();

        log::info!("{}", string.to_rust_string_lossy(try_catch_scope));
        stdout().flush().unwrap();
    }
    // TODO: Expose the world object and add methods for adding
    // https://github.com/denoland/deno/blob/main/core/bindings.rs#L463
    pub fn register_callback(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let state_rc = V8ScriptingEngine::state(scope);
        let mut state = state_rc.borrow_mut();
        let string = args.get(0).to_rust_string_lossy(scope);
        let function = match v8::Local::<v8::Function>::try_from(args.get(1)) {
            Ok(callback) => callback,
            Err(err) => {
                return;
            }
        };
        log::info!("added callback:{}", string);
        state
            .callbacks
            .insert(string, v8::Global::new(scope, function));
    }
}
#[cfg(target_arch = "wasm32")]
impl ScriptingFunctions {
    fn register_callback(&self, function: &js_sys::Function) {}
}
