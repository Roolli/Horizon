// use std::ops::Deref;

// use specs::{ReadExpect, ReadStorage, System, WriteStorage};

// use crate::{components::transform::Transform, renderer::state::State};

// pub struct LightTransform;
// impl<'a> System<'a> for LightTransform {
//     type SystemData = (ReadStorage<'a, LightHandle>, WriteStorage<'a, Transform>);

//     fn run(&mut self, mut data: Self::SystemData) {
//         use specs::Join;
//         let (light_handles, mut transforms) = data;

//         for (i, light) in (&light_handles, &mut transforms).join() {
//             let old_light_pos: glm::Vec3 = light.position;
//             light.position = glm::rotate_y_vec3(&old_light_pos, f32::to_radians(1.0f32 / 16.67));
//         }
//     }
// }
