use crate::components::scriptingcallback::ScriptingCallback;
use crate::components::transform::Transform;
use crate::renderer::utils::ecscontainer::ECSContainer;

use super::lifecycleevents::LifeCycleEvent;
#[cfg(not(target_arch = "wasm32"))]
use super::scriptingengine::V8ScriptingEngine;
use super::util::entityinfo::EntityInfo;
#[cfg(not(target_arch = "wasm32"))]
use rusty_v8 as v8;
use specs::prelude::*;
use specs::world::Generation;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::{convert::TryFrom, io::stdout};
#[cfg(not(target_arch = "wasm32"))]
use v8::{Function, Global};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]

pub struct ScriptingFunctions;

// ! https://github.com/rustwasm/wasm-bindgen/issues/858 might need JsValue instead of function

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ScriptingFunctions {
    pub fn register_callback(event_type: LifeCycleEvent, callback: js_sys::Function) {
        let ecs = ECSContainer::global_mut();
        let builder = ecs.world.create_entity();
        log::info!("{:?}", callback);
        callback.call0(&JsValue::NULL).unwrap();
        builder
            .with(ScriptingCallback::new(callback))
            .with(event_type)
            .build();
    }
    pub fn create_entity(entity_info: &wasm_bindgen::JsValue) -> wasm_bindgen::JsValue {
        let entity_info: EntityInfo = entity_info.into_serde().unwrap();
        let ecs = ECSContainer::global_mut();
        let builder = ecs.world.create_entity();
        let mut transform: Transform;
        for component in entity_info.components {
            match component.component_type.as_str() {
                "transform" => {
                    // transform = Transform::new(
                    //     component.position.unwrap().into(),
                    //     component.rotation.unwrap().into(),
                    //     component.scale.unwrap().into(),
                    // );
                }
                "physics" => {}
                "pointLight" => {}
                _ => {}
            }
        }

        JsValue::from_f64(builder.build().id().into())
    }
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
