// use specs::{Join, ReadExpect, ReadStorage, System};

// use crate::{
//     components::{
//         physicshandle::PhysicsHandle,
//         transform::{Transform, TransformRaw},
//     },
//     renderer::state::State,
//     resources::bindingresourcecontainer::BindingResourceContainer,
// };

// pub struct UpdateUniformBuffers;

// impl<'a> System<'a> for UpdateUniformBuffers {
//     type SystemData = (
//         ReadStorage<'a, Transform>,
//         ReadStorage<'a, PhysicsHandle>,
//         ReadExpect<'a, State>,
//         ReadExpect<'a, BindingResourceContainer>,
//     );

//     fn run(&mut self, data: Self::SystemData) {
//         let (transforms, handles, state, resource_container) = data;
//         let instance_data = (&transforms, &handles)
//             .join()
//             .map(|(transform, _handle)| Transform::to_raw(transform))
//             .collect::<Vec<_>>();
//     }
// }
