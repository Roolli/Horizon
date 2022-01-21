
use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect};

use crate::{
    renderer::{
        primitives::{
            lights::{
                directionallight::DirectionalLight, pointlight::PointLight, spotlight::SpotLight,
            },
            uniforms::{Globals},
        },
        state::State,
    },
    resources::{bindingresourcecontainer::BindingResourceContainer, camera::Camera},
};

pub struct UpdateBuffers;

impl<'a> System<'a> for UpdateBuffers {
    type SystemData = (
        ReadExpect<'a, BindingResourceContainer>,
        ReadExpect<'a, State>,
        ReadExpect<'a, DirectionalLight>,
        WriteExpect<'a, Globals>,
        ReadExpect<'a, Camera>,
        ReadStorage<'a, PointLight>,
        ReadStorage<'a, SpotLight>,
    );

    fn run(
        &mut self,
        (binding_resource_container, state, dir_light, mut globals, cam,point_lights,spot_lights): Self::SystemData,
    ) {
        globals.update_view_proj_matrix(&cam);
        state.queue.write_buffer(
            binding_resource_container
                .buffers
                .get("uniform_buffer")
                .unwrap(),
            0,
            bytemuck::bytes_of(&*globals),
        );
        state.queue.write_buffer(
            binding_resource_container
                .buffers
                .get("directional_light_buffer")
                .unwrap(),
            0,
            bytemuck::bytes_of(&dir_light.to_raw(&cam)),
        );
        //TODO: optimize to minimize copying
        let point_light_raw = point_lights
            .join()
            .map(PointLight::to_raw)
            .collect::<Vec<_>>();
        let spot_light_raw = spot_lights
            .join()
            .map(SpotLight::to_raw)
            .collect::<Vec<_>>();
        globals.set_point_light_count(point_light_raw.len() as u32);
        globals.set_spot_light_count(spot_light_raw.len() as u32);
        state.queue.write_buffer(
            binding_resource_container
                .buffers
                .get("spot_light_buffer")
                .unwrap(),
            0,
            bytemuck::cast_slice(&spot_light_raw),
        );
        state.queue.write_buffer(
            binding_resource_container
                .buffers
                .get("point_light_buffer")
                .unwrap(),
            0,
            bytemuck::cast_slice(&point_light_raw),
        );
        //TODO: move to a system which handles resizing to reduce unneeded copies.
    }
}
