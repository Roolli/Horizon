use std::ops::Deref;

use specs::{ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    components::transform::Transform,
    renderer::{light::Light, state::State},
};

pub struct LightTransform;
impl<'a> System<'a> for LightTransform {
    type SystemData = WriteStorage<'a, Light>;

    fn run(&mut self, mut data: Self::SystemData) {
        use specs::Join;

        for (i, light) in (&mut data).join().enumerate() {
            let old_light_pos: glm::Vec4 = light.position.into();
            light.position = if i % 2 == 0 {
                glm::rotate_y_vec4(&old_light_pos, f32::to_radians(1.0f32)).into()
            } else {
                glm::rotate_y_vec4(&old_light_pos, f32::to_radians(-1.0f32)).into()
            }
        }
    }
}
