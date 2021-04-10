use std::{borrow::Borrow, collections::HashMap, sync::Once};

use rusty_v8 as v8;
use v8::{
    Context, ContextScope, CreateParams, HandleScope, Local, OwnedIsolate, Platform, UniquePtr,
    UniqueRef,
};

use super::scriptingfunctions::ScriptingFunctions;
static PLATFORM_INIT: Once = Once::new();
pub struct V8ScriptingEngine {
    pub isolate: OwnedIsolate,
    pub global_context: v8::Global<v8::Context>,
}

fn platform_init() {
    PLATFORM_INIT.call_once(|| {
        let platform = v8::new_default_platform().unwrap();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

impl V8ScriptingEngine {
    pub fn new() -> Self {
        platform_init();
        let mut isolate = v8::Isolate::new(Default::default());
        let global_context;
        {
            let handle_scope = &mut v8::HandleScope::new(&mut isolate);
            let context = Self::setup_global_functions(handle_scope);

            global_context = v8::Global::new(handle_scope, context);
        }
        Self {
            global_context,
            isolate,
        }
    }
    pub fn setup_global_functions<'s>(
        scope: &mut v8::HandleScope<'s, ()>,
    ) -> v8::Local<'s, v8::Context> {
        let scope = &mut v8::EscapableHandleScope::new(scope);
        let context = v8::Context::new(scope);
        let global = context.global(scope);

        let scope = &mut v8::ContextScope::new(scope, context);

        let horizon_key = v8::String::new(scope, "Horizon").unwrap();
        let horizon_val = v8::Object::new(scope);
        global.set(scope, horizon_key.into(), horizon_val.into());
        Self::bind_function(scope, horizon_val, "print", ScriptingFunctions::print);
        Self::bind_function(
            scope,
            horizon_val,
            "registerCallback",
            ScriptingFunctions::register_callback,
        );
        scope.escape(context)
    }
    fn script_origin<'a>(
        scope: &mut v8::HandleScope<'a>,
        resource_name: v8::Local<'a, v8::String>,
    ) -> v8::ScriptOrigin<'a> {
        let source_map = v8::String::new(scope, "").unwrap();
        v8::ScriptOrigin::new(
            scope,
            resource_name.into(),
            0,
            0,
            false,
            123,
            source_map.into(),
            false,
            false,
            false,
        )
    }

    fn module_origin<'a>(
        scope: &mut v8::HandleScope<'a>,
        resource_name: v8::Local<'a, v8::String>,
    ) -> v8::ScriptOrigin<'a> {
        let source_map = v8::String::new(scope, "").unwrap();
        v8::ScriptOrigin::new(
            scope,
            resource_name.into(),
            0,
            0,
            false,
            123,
            source_map.into(),
            false,
            false,
            true,
        )
    }
    pub fn execute(&mut self, js_filename: &str, js_src: &str) {
        let context = self.global_context.clone();

        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, context);
        let source = v8::String::new(scope, js_src).unwrap();
        let name = v8::String::new(scope, js_filename).unwrap();
        let origin = Self::script_origin(scope, name);

        let tc_scope = &mut v8::TryCatch::new(scope);

        let script = v8::Script::compile(tc_scope, source, Some(&origin)).unwrap();
        match script.run(tc_scope) {
            Some(_) => {}
            None => {
                let exception = tc_scope.exception().unwrap();
                let message = v8::Exception::create_message(tc_scope, exception);
                let message_string = message.get(tc_scope);
                log::error!(
                    "exception has occured: {}",
                    message_string.to_rust_string_lossy(tc_scope)
                );
            }
        }
    }

    #[inline(always)]
    fn bind_function(
        scope: &mut v8::HandleScope<'_>,
        obj: v8::Local<v8::Object>,
        name: &'static str,
        callback: impl v8::MapFnTo<v8::FunctionCallback>,
    ) {
        let key = v8::String::new(scope, name).unwrap();
        let template = v8::FunctionTemplate::new(scope, callback);
        let val = template.get_function(scope).unwrap();
        obj.set(scope, key.into(), val.into());
    }
}
