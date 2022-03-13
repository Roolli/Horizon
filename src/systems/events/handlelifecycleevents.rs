use specs::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
use crate::components::scriptingcallback::ScriptingCallback;
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
                    //callback.get_callback().call0(&JsValue::UNDEFINED).unwrap();
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

                //callback.get_callback().call1(&JsValue::NULL,&JsValue::from(dt.delta)).unwrap();
            }
        }
    }
}

pub struct HandleDestroyCallbacks; // Might not be needed as of now. perhaps some save/load mechanic could utilize this save all currently alive entities


pub struct OnEntityCollision; // might be managed by physics system.
