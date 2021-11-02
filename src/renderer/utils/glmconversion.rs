#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct HorizonVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HorizonVec3 {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
impl From<HorizonVec3> for glm::Vec3 {
    fn from(val: HorizonVec3) -> Self {
        glm::Vec3::new(val.x, val.y, val.z)
    }
}
