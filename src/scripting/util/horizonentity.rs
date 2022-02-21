use crate::components::componenttypes::ComponentTypes;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct HorizonEntity {
    entity_id: JsValue,
}
#[wasm_bindgen]
impl HorizonEntity {
    pub fn new(entity_id: JsValue) -> Self {
        HorizonEntity { entity_id }
    }

    #[wasm_bindgen(js_name = "getComponent")]
    pub fn get_component(&self, component_type: ComponentTypes) -> JsValue {
        JsValue::TRUE
        //TODO: add scripting function fn to get data back based on entity id.
    }
    #[wasm_bindgen(js_name = "setComponent")]
    pub fn set_component(&self, component_type: ComponentTypes) {}
    #[wasm_bindgen(js_name = "deleteComponent")]
    pub fn delete_component(&self, component_type: ComponentTypes) {}
    #[wasm_bindgen(js_name = "addComponent")]
    pub fn add_component(&self, component_type: ComponentTypes) {}
}
