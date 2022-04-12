use rapier3d::na::{Matrix3, Matrix4, Perspective3};
use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect};

use crate::components::transform::Transform;
use crate::renderer::primitives::uniforms::{LightCullingUniforms, SkyboxUniform};
use crate::{
    renderer::{
        primitives::{
            lights::{
                directionallight::DirectionalLight, pointlight::PointLight, spotlight::SpotLight,
            },
            uniforms::Globals,
        },
        state::State,
    },
    resources::{bindingresourcecontainer::BindingResourceContainer, camera::Camera},
    BufferTypes, Projection, Skybox, Uniform,
};

pub struct UpdateBuffers;

impl<'a> System<'a> for UpdateBuffers {
    type SystemData = (
        ReadExpect<'a, BindingResourceContainer>,
        ReadExpect<'a, State>,
        ReadExpect<'a, DirectionalLight>,
        WriteExpect<'a, Globals>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a, Projection>,
        ReadStorage<'a, PointLight>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, SpotLight>,
    );

    fn run(
        &mut self,
        (
            binding_resource_container,
            state,
            dir_light,
            mut globals,
            cam,
            proj,
            point_lights,
            transforms,
            spot_lights,
        ): Self::SystemData,
    ) {
        globals.update_view_proj_matrix(&cam, &proj);

        //get 3x3 matrix and remove translation & keep yaw only
        let view_matrix = cam.get_view_matrix();

        let removed_translation: Matrix4<f32> = Matrix4::from_data(
            Matrix3::new(
                view_matrix.m11,
                view_matrix.m12,
                view_matrix.m13,
                view_matrix.m21,
                view_matrix.m22,
                view_matrix.m23,
                view_matrix.m31,
                view_matrix.m32,
                view_matrix.m33,
            )
            .to_homogeneous()
            .data,
        );
        state.queue.write_buffer(
            binding_resource_container.buffers[Skybox].as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&SkyboxUniform {
                view: (Matrix4::from(State::OPENGL_TO_WGPU_MATRIX) * removed_translation).into(),
                projection_inverse: nalgebra_glm::reversed_perspective_rh_zo(proj.aspect_ratio,proj.fov_y,proj.z_near,100.0).try_inverse().unwrap().into(),
            }),
        );
        state.queue.write_buffer(
            binding_resource_container.buffers[BufferTypes::DirectionalLight]
                .as_ref()
                .unwrap(),
            0,
            bytemuck::bytes_of(&dir_light.to_raw()),
        );
        let point_light_raw = (&transforms, &point_lights)
            .join()
            .map(|(transform, pl)| {
                if let Some(attached_id) = pl.attached_to {
                    if let Some(ent) = transforms.get(attached_id) {
                        pl.to_raw(ent.position)
                    } else {
                        pl.to_raw(transform.position)
                    }
                } else {
                    pl.to_raw(transform.position)
                }
            })
            .collect::<Vec<_>>();
        let spot_light_raw = spot_lights
            .join()
            .map(SpotLight::to_raw)
            .collect::<Vec<_>>();
        globals.set_point_light_count(point_light_raw.len() as u32);
        globals.set_spot_light_count(spot_light_raw.len() as u32);
        state.queue.write_buffer(
            binding_resource_container.buffers[BufferTypes::LightCulling]
                .as_ref()
                .unwrap(),
            0,
            bytemuck::bytes_of(&LightCullingUniforms::new(
                &Projection::new(
                    state.sc_descriptor.width,
                    state.sc_descriptor.height,
                    f32::to_radians(45.0),
                    0.1,
                ),
                &cam,
            )),
        );
        state.queue.write_buffer(
            binding_resource_container.buffers[BufferTypes::SpotLight]
                .as_ref()
                .unwrap(),
            0,
            bytemuck::cast_slice(&spot_light_raw),
        );
        state.queue.write_buffer(
            binding_resource_container.buffers[BufferTypes::PointLight]
                .as_ref()
                .unwrap(),
            0,
            bytemuck::cast_slice(&point_light_raw),
        );
        state.queue.write_buffer(
            binding_resource_container.buffers[Uniform]
                .as_ref()
                .unwrap(),
            0,
            bytemuck::bytes_of(&*globals),
        );
    }
}
