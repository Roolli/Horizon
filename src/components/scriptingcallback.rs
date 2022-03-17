use specs::*;


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: js_sys::Function,
}

#[cfg(target_arch = "wasm32")]
impl ScriptingCallback {
    pub fn new(callback: js_sys::Function) -> Self {
        Self { callback }
    }
    pub fn get_callback(&self) ->&js_sys::Function
    {
        &self.callback
    }
}
#[cfg(target_arch = "wasm32")]
impl ExecuteFunction for ScriptingCallback {
    fn execute_with_no_args(&self) {
        self.get_callback().call0(&JsValue::UNDEFINED).unwrap();
    }

    fn execute_with_args(&self,args: Vec<f32>) {
        self.get_callback().call1(&JsValue::NULL,&JsValue::from(args[0])).unwrap();
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: rusty_v8::Global<rusty_v8::Function>,
}
#[cfg(not(target_arch = "wasm32"))]
impl ExecuteFunction for ScriptingCallback
{
    fn execute_with_no_args(&self) {
        todo!()
    }

    fn execute_with_args(&self, args: Vec<f32>) {
        todo!()
    }
}


pub trait ExecuteFunction
{
    fn execute_with_no_args(&self);

    // use numbers for now might change to a boxed value
    fn execute_with_args(&self,args:Vec<f32>);
}