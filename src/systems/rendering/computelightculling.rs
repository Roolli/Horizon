use std::convert::TryInto;

use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect};

use crate::{
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::{
            lighting::LightBindGroup, tiling::TilingBindGroup, uniforms::UniformBindGroup,
        },
        pipelines::lightcullingpipeline::LightCullingPipeline,
        state::State,
    },
    resources::commandencoder::HorizonCommandEncoder,
    BindingResourceContainer, BufferTypes,
};

pub struct ComputeLightCulling;

impl<'a> System<'a> for ComputeLightCulling {
    type SystemData = (
        WriteExpect<'a, HorizonCommandEncoder>,
        ReadExpect<'a, State>,
        ReadStorage<'a, LightBindGroup>,
        ReadStorage<'a, UniformBindGroup>,
        ReadStorage<'a, BindGroupContainer>,
        ReadExpect<'a, LightCullingPipeline>,
        ReadStorage<'a, TilingBindGroup>,
        ReadExpect<'a, BindingResourceContainer>,
    );

    fn run(
        &mut self,
        (
            mut cmd_encoder,
            state,
            light_bind_group,
            uniform_bind_group,
            bind_group_container,
            pipeline,
            tiling_bind_group,
            binding_resource_container,
        ): Self::SystemData,
    ) {
        let command_encoder = cmd_encoder.get_encoder();
        command_encoder.clear_buffer(
            binding_resource_container.buffers[BufferTypes::LightId]
                .as_ref()
                .unwrap(),
            0,
            None,
        );
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Light Culling pass"),
        });
        let (_, uniform_bind_group_container) = (&uniform_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();
        let (_, light_bind_group_container) = (&light_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();
        let (_, tiling_bind_group_container) = (&tiling_bind_group, &bind_group_container)
            .join()
            .next()
            .unwrap();

        compute_pass.set_pipeline(&pipeline.0);
        compute_pass.set_bind_group(0, &light_bind_group_container.bind_group, &[]);
        compute_pass.set_bind_group(1, &uniform_bind_group_container.bind_group, &[]);
        compute_pass.set_bind_group(2, &tiling_bind_group_container.bind_group, &[]);
        compute_pass.dispatch(
            (f32::ceil(State::MAX_POINT_LIGHTS as f32 / 64.0)) as u32,
            1,
            1,
        );
        drop(compute_pass);

        cmd_encoder.finish(&state.device, &state.queue);
    }
}
