use std::ops::Deref;

use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    renderer::{
        bindgroupcontainer::BindGroupContainer,
        bindgroups::{deferred::DeferredBindGroup, gbuffer::GBuffer, HorizonBindGroup},
        primitives::{texture::Texture, uniforms::CanvasConstants},
        state::State,
    },
    resources::{bindingresourcecontainer::BindingResourceContainer, windowevents::ResizeEvent},
};

pub struct Resize;

impl<'a> System<'a> for Resize {
    type SystemData = (
        WriteExpect<'a, ResizeEvent>,
        WriteExpect<'a, State>,
        WriteExpect<'a, BindingResourceContainer>,
        WriteStorage<'a, BindGroupContainer>,
        ReadStorage<'a, DeferredBindGroup>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let mut state = data.1;
        let mut resize_event = data.0;
        let mut resource_container = data.2;
        let mut bind_group_container = data.3;
        let deferred_bind_group = data.4;
        let entites = data.5;
        if resize_event.handled {
            return;
        }

        state.size = resize_event.new_size;
        state.sc_descriptor.height = resize_event.new_size.height;
        state.sc_descriptor.width = resize_event.new_size.width;
        state.depth_texture =
            Texture::create_depth_texture(&state.device, &state.sc_descriptor, "depth_texture");
        GBuffer::generate_g_buffers(&state.device, &state.sc_descriptor, &mut resource_container);
        let (_, _, entity) = (&deferred_bind_group, &bind_group_container, &entites)
            .join()
            .next()
            .unwrap();
        let bind_group = bind_group_container.get_mut(entity).unwrap();
        state.surface.configure(&state.device, &state.sc_descriptor);
        state.queue.write_buffer(
            resource_container
                .buffers
                .get("canvas_size_buffer")
                .unwrap(),
            0,
            bytemuck::bytes_of(&CanvasConstants {
                size: [
                    state.sc_descriptor.width as f32,
                    state.sc_descriptor.height as f32,
                ],
            }),
        );
        *bind_group = DeferredBindGroup::create_container(
            &state.device,
            (
                resource_container.samplers.get("texture_sampler").unwrap(),
                resource_container
                    .texture_views
                    .get("position_view")
                    .unwrap(),
                resource_container.texture_views.get("normal_view").unwrap(),
                resource_container.texture_views.get("albedo_view").unwrap(),
                resource_container
                    .buffers
                    .get("canvas_size_buffer")
                    .unwrap(),
            ),
        );
        log::info!("resize has occured!");
        resize_event.handled = true;
    }
}
