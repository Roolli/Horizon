use rapier3d::na::{Matrix3, Matrix4};
use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect};

use crate::{Projection, renderer::{
    primitives::{
        lights::{
            directionallight::DirectionalLight, pointlight::PointLight, spotlight::SpotLight,
        },
        uniforms::{Globals},
    },
    state::State,
}, resources::{bindingresourcecontainer::BindingResourceContainer, camera::Camera}};
use crate::renderer::primitives::uniforms::SkyboxUniform;

pub struct UpdateBuffers;

impl<'a> System<'a> for UpdateBuffers {
    type SystemData = (
        ReadExpect<'a, BindingResourceContainer>,
        ReadExpect<'a, State>,
        ReadExpect<'a, DirectionalLight>,
        WriteExpect<'a, Globals>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a,Projection>,
        ReadStorage<'a, PointLight>,
        ReadStorage<'a, SpotLight>,
    );

    fn run(
        &mut self,
        (binding_resource_container, state, dir_light, mut globals, cam,proj,point_lights,spot_lights): Self::SystemData,
    ) {
        globals.update_view_proj_matrix(&cam,&proj);
        state.queue.write_buffer(
            binding_resource_container
                .buffers
                .get("uniform_buffer")
                .unwrap(),
            0,
            bytemuck::bytes_of(&*globals),
        );
        // get 3x3 matrix and remove translation
        // Matrix4::from_data(Matrix3::new(view_matrix.m11,view_matrix.m12,view_matrix.m13,view_matrix.m21,view_matrix.m22,view_matrix.m23,view_matrix.m31,view_matrix.m32,view_matrix.m33).to_homogeneous().data)).into()
         let view_matrix = cam.get_view_matrix();
        state.queue.write_buffer(
            binding_resource_container.buffers.get("skybox_buffer").unwrap(),
            0,
            bytemuck::bytes_of(
                &SkyboxUniform{
                    view: (Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * view_matrix).into(),
                    projection_inverse: proj.calc_proj_matrix().try_inverse().unwrap().into()
                }));
        state.queue.write_buffer(
            binding_resource_container
                .buffers
                .get("directional_light_buffer")
                .unwrap(),
            0,
            bytemuck::bytes_of(&dir_light.to_raw(&cam,&proj)),
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
