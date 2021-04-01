use specs::{ReadExpect, System, WriteExpect};

use crate::{
    renderer::{primitives::texture::Texture, state::State},
    resources::windowevents::ResizeEvent,
};

pub struct Resize;

impl<'a> System<'a> for Resize {
    type SystemData = (WriteExpect<'a, ResizeEvent>, WriteExpect<'a, State>);

    fn run(&mut self, data: Self::SystemData) {
        let mut state = data.1;
        let mut resize_event = data.0;
        if resize_event.handled {
            return;
        }
        state.size = resize_event.new_size;
        state.sc_descriptor.height = resize_event.new_size.height;
        state.sc_descriptor.width = resize_event.new_size.width;
        state.depth_texture =
            Texture::create_depth_texture(&state.device, &state.sc_descriptor, "depth_texture");
        state.swap_chain = state
            .device
            .create_swap_chain(&state.surface, &state.sc_descriptor);
        resize_event.handled = true;
    }
}
