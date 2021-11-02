use specs::{ReadExpect, System, WriteExpect};

use crate::scripting::scriptingfunctions::LIFECYCLE_EVENTS;
use crate::{renderer::state::State, resources::eguirenderpass::EguiRenderPass};
pub struct RenderUIPass;
use wasm_bindgen::prelude::*;

impl<'a> System<'a> for RenderUIPass {
    type SystemData = (ReadExpect<'a, State>, WriteExpect<'a, EguiRenderPass>);

    fn run(&mut self, data: Self::SystemData) {
        LIFECYCLE_EVENTS.with(|v| {
            for functions in v.borrow().values() {
                for f in functions {
                    if let Err(e) = f.call1(&JsValue::undefined(), &JsValue::from_f64(42f64)) {
                        // TODO: handle err,
                    }
                }
            }
        });
    }
}
