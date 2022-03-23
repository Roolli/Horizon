use crate::components::scriptingcallback::{CallbackArgs, ExecuteFunction, ScriptingCallback};
use crate::scripting::scriptevent::ScriptEvent;
use crate::{DeltaTime, HorizonScriptingEngine};
use specs::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// executes all callbacks BEFORE the event loop is started.
pub struct HandleInitCallbacks;

impl<'a> System<'a> for HandleInitCallbacks {
    type SystemData = (
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(&mut self, (callbacks, events, mut scripting): Self::SystemData) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::Init {
                callback.execute_with_no_args(&mut scripting);
            }
        }
    }
}

pub struct HandleOnRenderCallbacks;

impl<'a> System<'a> for HandleOnRenderCallbacks {
    type SystemData = (
        ReadExpect<'a, DeltaTime>,
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(&mut self, (dt, callbacks, events, mut scripting_engine): Self::SystemData) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::OnTick {
                callback.execute_with_args(&mut scripting_engine, CallbackArgs::Tick(dt.delta));
            }
        }
    }
}

pub struct HandleDestroyCallbacks; // Might not be needed as of now. perhaps some save/load mechanic could utilize this save all currently alive entities

pub struct OnEntityCollision; // might be managed by physics system.
