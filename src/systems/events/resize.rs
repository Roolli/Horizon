

use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{CanvasSize, DeferredAlbedo, DeferredNormals, DeferredPosition, DeferredTexture, Projection, renderer::{
    bindgroupcontainer::BindGroupContainer,
    bindgroups::{deferred::DeferredBindGroup, gbuffer::GBuffer, HorizonBindGroup},
    primitives::{texture::Texture, uniforms::CanvasConstants},
    state::State,
}, resources::{bindingresourcecontainer::BindingResourceContainer, windowevents::ResizeEvent}};use crate::resources::commandencoder::HorizonCommandEncoder;
use crate::TextureViewTypes::DeferredSpecular;

pub struct Resize;

impl<'a> System<'a> for Resize {
    type SystemData = (
        WriteExpect<'a, ResizeEvent>,
        WriteExpect<'a, State>,
        WriteExpect<'a, BindingResourceContainer>,
        WriteStorage<'a, BindGroupContainer>,
        WriteExpect<'a,Projection>,
        ReadStorage<'a, DeferredBindGroup>,
        WriteExpect<'a, HorizonCommandEncoder>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let mut state = data.1;
        let mut resize_event = data.0;
        let mut resource_container = data.2;
        let mut bind_group_container = data.3;
        let mut proj = data.4;
        let deferred_bind_group = data.5;
        let entities = data.7;
        if resize_event.handled {
            return;
        }

        state.size = resize_event.new_size;
        state.sc_descriptor.height = resize_event.new_size.height;
        state.sc_descriptor.width = resize_event.new_size.width;
        proj.resize(resize_event.new_size.width,resize_event.new_size.height);
        if let Some(new_scale) =resize_event.scale_factor
        {
            state.scale_factor = new_scale;
        }
        state.depth_texture =
            Texture::create_depth_texture(&state.device, &state.sc_descriptor, "depth_texture");
        GBuffer::generate_g_buffers(&state.device, &state.sc_descriptor, &mut resource_container);
        let (_, _, entity) = (&deferred_bind_group, &bind_group_container, &entities)
            .join()
            .next()
            .unwrap();
        let bind_group = bind_group_container.get_mut(entity).unwrap();
        state.surface.configure(&state.device, &state.sc_descriptor);
        state.queue.write_buffer(
            resource_container
                .buffers[CanvasSize].as_ref().unwrap(),
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
                resource_container.samplers[DeferredTexture].as_ref().unwrap(),
                resource_container
                    .texture_views[DeferredPosition].as_ref().unwrap(),
                resource_container.texture_views[DeferredNormals].as_ref().unwrap(),
                resource_container.texture_views[DeferredAlbedo].as_ref().unwrap(),
                resource_container.texture_views[DeferredSpecular].as_ref().unwrap(),
                resource_container
                    .buffers[CanvasSize].as_ref().unwrap()
            ),
        );
        log::info!("resize has occured!");

        resize_event.handled = true;
    }
}
