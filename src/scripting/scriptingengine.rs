use std::{sync::Once};

#[cfg(not(target_arch = "wasm32"))]
use rusty_v8 as v8;
#[cfg(not(target_arch = "wasm32"))]
use v8::{
    Context, ContextScope, CreateParams, HandleScope, Local, OwnedIsolate, Platform, UniquePtr,
    UniqueRef,
};



static PLATFORM_INIT: Once = Once::new();
#[cfg(not(target_arch = "wasm32"))]
pub struct V8ScriptingEngine {
    pub isolate: OwnedIsolate,
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_init() {
    PLATFORM_INIT.call_once(|| {
        let platform = v8::new_default_platform().unwrap();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}
#[cfg(not(target_arch = "wasm32"))]
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
        isolate.set_slot(Rc::new(RefCell::new(ScriptingEngineState {
            callbacks: HashMap::new(),
            global_context: Some(global_context),
        })));
        Self { isolate }
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
    pub fn global_context(&mut self) -> v8::Global<v8::Context> {
        let state = Self::state(&self.isolate);
        let state = state.as_ref().borrow();
        state.global_context.clone().unwrap()
    }
    pub fn execute(&mut self, js_filename: &str, js_src: &str) {
        let global_context = self.global_context();
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, global_context);
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
    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<ScriptingEngineState>> {
        let state = isolate
            .get_slot::<Rc<RefCell<ScriptingEngineState>>>()
            .unwrap();
        state.clone()
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub struct ScriptingEngineState {
    pub global_context: Option<v8::Global<v8::Context>>,
    pub callbacks: HashMap<String, v8::Global<v8::Function>>,
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



