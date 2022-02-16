use specs::{System, Write};
use crate::{CameraController};

use crate::resources::windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent};

pub struct HandleWindowEvents;

impl<'a> System<'a> for HandleWindowEvents {
    type SystemData = (
        Write<'a, MouseInputEvent>,
        Write<'a, MouseMoveEvent>,
        Write<'a, KeyboardEvent>,
        Write<'a,CameraController>
    );

    fn run(
        &mut self,
        (mut mouse_input_event, mut mouse_move_event, mut keyboard_event,mut cam_controller): Self::SystemData,
    ) {
        //TODO: iterate over all subscribed event handlers and invoke them with the appropriate data
        if !mouse_input_event.handled {
            mouse_input_event.handled = true;
        }
        if !mouse_move_event.handled {
            cam_controller.handle_mouse_move(&mouse_move_event);
            mouse_move_event.handled = true;
        }
        if !keyboard_event.handled {
            cam_controller.handle_keyboard_event(&keyboard_event.info);
            keyboard_event.handled = true;
        }
    }
}
