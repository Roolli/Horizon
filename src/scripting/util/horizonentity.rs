use crate::components::componenttypes::ComponentTypes;
use crate::scripting::scriptingfunctions::ScriptingFunctions;
use crate::scripting::util::entityinfo::EntityInfo;
use serde::Deserialize;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonEntity {
    entity_id: u32,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HorizonEntity {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(entity_info: &wasm_bindgen::JsValue) -> Result<HorizonEntity, JsValue> {
        let entity_info: EntityInfo = entity_info.into_serde().unwrap();
        ScriptingFunctions::create_entity(entity_info).map_err(|e| {
            JsValue::from_str(format!("Entity create failed with error: {:?}", e).as_str())
        })
    }

    pub fn from_entity_id(entity_id: u32) -> Self {
        HorizonEntity { entity_id }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getComponent"))]
    pub fn get_component(&self, component_type: ComponentTypes) -> JsValue {
        ScriptingFunctions::get_component(component_type, self.entity_id)
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "setComponent"))]
    pub fn set_component(&self, component_data: &JsValue) {
        ScriptingFunctions::insert_component(component_data.into_serde().unwrap(), self.entity_id)
            .unwrap(); //TODO: return err
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "deleteComponent"))]
    pub fn delete_component(&self, component_type: ComponentTypes) {
        ScriptingFunctions::delete_component(component_type, self.entity_id);
    }
}
