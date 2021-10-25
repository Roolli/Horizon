use specs::{System, Write};

use crate::resources::windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent};

pub struct HandleWindowEvents;

impl<'a> System<'a> for HandleWindowEvents {
    type SystemData = (
        Write<'a, MouseInputEvent>,
        Write<'a, MouseMoveEvent>,
        Write<'a, KeyboardEvent>,
    );

    fn run(
        &mut self,
        (mut mouse_input_event, mut mouse_move_event, mut keyboard_event): Self::SystemData,
    ) {
        //TODO: iterate over all subscribed event handlers and invoke them with the appropriate data
        if !mouse_input_event.handled {
            mouse_input_event.handled = true;
        }
        if !mouse_move_event.handled {
            mouse_move_event.handled = true;
        }
        if !keyboard_event.handled {
            keyboard_event.handled = true;
        }
    }
}
