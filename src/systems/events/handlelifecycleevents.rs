use specs::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::components::scriptingcallback::{ExecuteFunction, ScriptingCallback};
use crate::DeltaTime;
use crate::scripting::scriptevent::ScriptEvent;

/// executes all callbacks BEFORE the event loop is started.
pub struct HandleInitCallbacks;

impl<'a> System<'a> for HandleInitCallbacks {
    type SystemData = (ReadStorage<'a, ScriptingCallback>,ReadStorage<'a,ScriptEvent>);

    fn run(&mut self, (callbacks,events): Self::SystemData) {
            for( event,callback) in (&events,&callbacks).join() {
                if *event == ScriptEvent::Init
                {
                    callback.execute_with_no_args();
                }
            }
    }
}

pub struct HandleOnRenderCallbacks;

impl<'a> System<'a> for HandleOnRenderCallbacks{
    type SystemData = (ReadExpect<'a,DeltaTime>,ReadStorage<'a,ScriptingCallback>,ReadStorage<'a,ScriptEvent>);

    fn run(&mut self, (dt,callbacks,events): Self::SystemData) {

        for (event,callback) in(&events, &callbacks).join(){
            if *event == ScriptEvent::OnTick {
                callback.execute_with_args(vec![dt.delta]);
            }
        }
    }
}

pub struct HandleDestroyCallbacks; // Might not be needed as of now. perhaps some save/load mechanic could utilize this save all currently alive entities


pub struct OnEntityCollision; // might be managed by physics system.
