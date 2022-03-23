use anyhow::Error;
use deno_core::error::generic_error;
use deno_core::{
    Extension, JsRuntime, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier,
    ModuleType, OpState, RuntimeOptions,
};
use std::pin::Pin;

// use std::collections::HashMap;
// use std::sync::Once;
//
// use crate::components::scriptingcallback::ScriptingCallback;
// use crate::scripting::scriptevent::ScriptEvent;
// use crate::ECSContainer;
// use specs::{Builder, WorldExt};
// #[cfg(not(target_arch = "wasm32"))]
// use v8;
// #[cfg(not(target_arch = "wasm32"))]
// use v8::{
//     Context, ContextScope, CreateParams, HandleScope, Local, OwnedIsolate, Platform, UniquePtr,
//     UniqueRef,
// };
//
// static PLATFORM_INIT: Once = Once::new();
// #[cfg(not(target_arch = "wasm32"))]
// pub struct V8ScriptingEngine {
//     pub isolate: OwnedIsolate,
// }
//
// #[cfg(not(target_arch = "wasm32"))]
// fn platform_init() {
//     PLATFORM_INIT.call_once(|| {
//         let platform = v8::new_default_platform(0, false).make_shared();
//         v8::V8::initialize_platform(platform);
//         v8::V8::initialize();
//     });
// }
// #[cfg(not(target_arch = "wasm32"))]
// impl V8ScriptingEngine {
//     pub fn new() -> Self {
//         platform_init();
//         let mut isolate = v8::Isolate::new(Default::default());
//         let global_context;
//         {
//             let handle_scope = &mut v8::HandleScope::new(&mut isolate);
//             let context = Self::setup_global_functions(handle_scope);
//
//             global_context = v8::Global::new(handle_scope, context);
//         }
//         isolate.set_slot(std::rc::Rc::new(std::cell::RefCell::new(
//             ScriptingEngineState {
//                 callbacks: HashMap::new(),
//                 global_context: Some(global_context),
//             },
//         )));
//         Self { isolate }
//     }
//     pub fn setup_global_functions<'s>(
//         scope: &mut v8::HandleScope<'s, ()>,
//     ) -> v8::Local<'s, v8::Context> {
//         let scope = &mut v8::EscapableHandleScope::new(scope);
//         let context = v8::Context::new(scope);
//         let global = context.global(scope);
//
//         let scope = &mut v8::ContextScope::new(scope, context);
//
//         let horizon_key = v8::String::new(scope, "Horizon").unwrap();
//         let horizon_val = v8::Object::new(scope);
//         global.set(scope, horizon_key.into(), horizon_val.into());
//         // Self::bind_function(scope, horizon_val, "print", ScriptingFunctions::print);
//         // Self::bind_function(
//         //     scope,
//         //     horizon_val,
//         //     "registerCallback",
//         //     ScriptingFunctions::register_callback,
//         // );
//         scope.escape(context)
//     }
//     fn script_origin<'a>(
//         scope: &mut v8::HandleScope<'a>,
//         resource_name: v8::Local<'a, v8::String>,
//     ) -> v8::ScriptOrigin<'a> {
//         let source_map = v8::String::new(scope, "").unwrap();
//         v8::ScriptOrigin::new(
//             scope,
//             resource_name.into(),
//             0,
//             0,
//             false,
//             123,
//             source_map.into(),
//             false,
//             false,
//             false,
//         )
//     }
//
//     fn module_origin<'a>(
//         scope: &mut v8::HandleScope<'a>,
//         resource_name: v8::Local<'a, v8::String>,
//     ) -> v8::ScriptOrigin<'a> {
//         let source_map = v8::String::new(scope, "").unwrap();
//         v8::ScriptOrigin::new(
//             scope,
//             resource_name.into(),
//             0,
//             0,
//             false,
//             123,
//             source_map.into(),
//             false,
//             false,
//             true,
//         )
//     }
//     pub fn global_context(&mut self) -> v8::Global<v8::Context> {
//         let state = Self::state(&self.isolate);
//         let state = state.as_ref().borrow();
//         state.global_context.clone().unwrap()
//     }
//     pub fn execute(&mut self, js_filename: &str, js_src: &str) {
//         let global_context = self.global_context();
//         let scope = &mut v8::HandleScope::with_context(&mut self.isolate, global_context);
//         let source = v8::String::new(scope, js_src).unwrap();
//         let name = v8::String::new(scope, js_filename).unwrap();
//         let origin = Self::script_origin(scope, name);
//
//         let tc_scope = &mut v8::TryCatch::new(scope);
//
//         let script = v8::Script::compile(tc_scope, source, Some(&origin)).unwrap();
//         match script.run(tc_scope) {
//             Some(_) => {}
//             None => {
//                 let exception = tc_scope.exception().unwrap();
//                 let message = v8::Exception::create_message(tc_scope, exception);
//                 let message_string = message.get(tc_scope);
//                 log::error!(
//                     "exception has occured: {}",
//                     message_string.to_rust_string_lossy(tc_scope)
//                 );
//             }
//         }
//     }
//
//     #[inline(always)]
//     fn bind_function(
//         scope: &mut v8::HandleScope<'_>,
//         obj: v8::Local<v8::Object>,
//         name: &'static str,
//         callback: impl v8::MapFnTo<v8::FunctionCallback>,
//     ) {
//         let key = v8::String::new(scope, name).unwrap();
//         let template = v8::FunctionTemplate::new(scope, callback);
//         let val = template.get_function(scope).unwrap();
//         obj.set(scope, key.into(), val.into());
//     }
//     pub fn state(isolate: &v8::Isolate) -> std::rc::Rc<std::cell::RefCell<ScriptingEngineState>> {
//         let state = isolate
//             .get_slot::<std::rc::Rc<std::cell::RefCell<ScriptingEngineState>>>()
//             .unwrap();
//         state.clone()
//     }
// }
// #[cfg(not(target_arch = "wasm32"))]
// pub struct ScriptingEngineState {
//     pub global_context: Option<v8::Global<v8::Context>>,
//     pub callbacks: HashMap<String, v8::Global<v8::Function>>,
// }
//
// #[cfg(not(target_arch = "wasm32"))]
// impl crate::scripting::scriptingfunctions::ScriptingFunctions {
//
//
//     // https://github.com/denoland/deno/blob/main/core/bindings.rs#L463
//     pub fn register_callback(
//         scope: &mut v8::HandleScope,
//         args: v8::FunctionCallbackArguments,
//         _rv: v8::ReturnValue,
//     ) {
//
// }

struct TimerPermission;

impl deno_web::TimersPermission for TimerPermission {
    fn allow_hrtime(&mut self) -> bool {
        true
    }

    fn check_unstable(&self, state: &OpState, api_name: &'static str) {}
}

#[derive(Default)]
struct ModLoader;

impl ModuleLoader for ModLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<ModuleSpecifier, Error> {
        let s = deno_core::resolve_import(specifier, referrer).unwrap();
        Ok(s)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            log::info!("{:?}", module_specifier);
            let module = ModuleSource {
                code: String::from(""),
                module_url_found: String::from(""),
                module_url_specified: String::from(""),
                module_type: ModuleType::JavaScript,
            };
            Ok(module)
        }
        .boxed_local()
    }
}

use crate::components::scriptingcallback::ScriptingCallback;
use crate::scripting::scriptevent::ScriptEvent;
use crate::scripting::util::horizonentity::HorizonEntity;
use crate::ECSContainer;
use deno_core::v8;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt};
use specs::{Builder, Join, WorldExt};

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct HorizonScriptingEngine;

#[cfg(not(target_arch = "wasm32"))]
pub struct HorizonScriptingEngine {
    pub js_runtime: JsRuntime,
}
#[cfg(not(target_arch = "wasm32"))]
impl Default for HorizonScriptingEngine {
    fn default() -> Self {
        let loader = std::rc::Rc::new(ModLoader::default());
        let extensions = Extension::builder()
            .ops(vec![op_load_model::decl(), op_model_exists::decl()])
            .build();
        let js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(loader),
            extensions: vec![
                deno_console::init(),
                deno_webidl::init(),
                deno_url::init(),
                deno_web::init::<TimerPermission>(BlobStore::default(), None),
                extensions,
            ],

            ..Default::default()
        });
        let mut runtime = Self { js_runtime };
        runtime.create_default_module();
        runtime
    }
}

impl HorizonScriptingEngine {
    fn create_default_module(&mut self) {
        let scope = &mut self.js_runtime.handle_scope();
        let global_context = scope.get_current_context().global(scope);
        let horizon_key = v8::String::new(scope, "HorizonInternal").unwrap();
        let horizon_val = v8::Object::new(scope);
        let callback_key = v8::String::new(scope, "registerCallback").unwrap();
        let print_key = v8::String::new(scope, "log").unwrap();
        let print_val = v8::Function::new(scope, Self::print_cb).unwrap();
        let func = v8::Function::new(scope, Self::register_callback_cb).unwrap();
        func.set_name(callback_key);
        global_context.set(scope, horizon_key.into(), horizon_val.into());
        horizon_val.set(scope, callback_key.into(), func.into());
        horizon_val.set(scope, print_key.into(), print_val.into());
    }
    pub fn print_cb(
        scope: &mut v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let obj = args.get(0);
        let try_catch_scope = &mut v8::TryCatch::new(scope);
        let string = obj.to_string(try_catch_scope).unwrap();

        log::info!("{}", string.to_rust_string_lossy(try_catch_scope));
    }

    fn register_callback_cb(
        scope: &mut deno_core::v8::HandleScope,
        args: v8::FunctionCallbackArguments,
        _rv: v8::ReturnValue,
    ) {
        let ecs = ECSContainer::global();
        let function = match v8::Local::<v8::Function>::try_from(args.get(0)) {
            Ok(callback) => callback,
            Err(err) => {
                log::info!("{:?}", err);
                return;
            }
        };

        let global_func: v8::Global<v8::Function> = v8::Global::new(scope, function);

        let builder = ecs.world.create_entity_unchecked();

        let event_type = ScriptEvent::from_number(args.get(1).int32_value(scope).unwrap());
        builder
            .with(ScriptingCallback::new(global_func))
            .with(event_type)
            .build();
    }
}
use crate::components::assetidentifier::AssetIdentifier;
use crate::scripting::scriptingfunctions::ScriptingFunctions;
use deno_core::op;
use deno_web::BlobStore;

#[op]
async fn op_load_model(model_name: String) -> Result<(), deno_core::anyhow::Error> {
    ScriptingFunctions::load_model(model_name).await;
    Ok(())
}
#[op]
fn op_model_exists(model_name: String) -> Result<Option<HorizonEntity>, deno_core::anyhow::Error> {
    let ecs = ECSContainer::global();
    let identifiers = ecs.world.read_component::<AssetIdentifier>();
    let ents = ecs.world.entities();
    for (ent, identifier) in (&ents, &identifiers).join() {
        if identifier.0 == model_name {
            return Ok(Some(HorizonEntity::from_entity_id(ent.id())));
        }
    }
    Ok(None)
}
