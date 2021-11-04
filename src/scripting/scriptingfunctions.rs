use super::lifecycleevents::LifeCycleEvent;
#[cfg(not(target_arch = "wasm32"))]
use super::scriptingengine::V8ScriptingEngine;
#[cfg(not(target_arch = "wasm32"))]
use rusty_v8 as v8;
use specs::storage::GenericWriteStorage;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::{convert::TryFrom, io::stdout};
#[cfg(not(target_arch = "wasm32"))]
use v8::{Function, Global};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
pub struct ScriptingFunctions;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ScriptingFunctions;

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
        let mut state_rc = V8ScriptingEngine::state(scope);
        let mut state = state_rc.borrow_mut();
        let string = args.get(0).to_rust_string_lossy(scope);
        let function = match v8::Local::<v8::Function>::try_from(args.get(1)) {
            Ok(callback) => callback,
            Err(err) => {
                return;
            }
        };
        log::info!("added callback:{}", string);
        // state
        //     .callbacks
        //     .insert(string, v8::Global::new(scope, function));
    }
}

// ! https://github.com/rustwasm/wasm-bindgen/issues/858 might need JsValue instead of function
// std::thread_local! {
//     pub static LIFECYCLE_EVENTS: RefCell<HashMap<LifeCycleEvent,Vec<js_sys::Function>>> = RefCell::new(HashMap::new());
// }

// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen]
// impl ScriptingFunctions {
//     fn register_callback(event_type: &LifeCycleEvent, function: js_sys::Function) {
//         LIFECYCLE_EVENTS.with(|v| {
//             if let Some(vec) = v.borrow_mut().get_mut(event_type) {
//                 vec.push(function);
//             }
//         });
//     }
// }
