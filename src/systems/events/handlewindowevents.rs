use crate::components::scriptingcallback::{CallbackArgs, ExecuteFunction, ScriptingCallback};
use crate::{CameraController, HorizonScriptingEngine};
use specs::{Join, Read, ReadStorage, System, Write, WriteExpect};
use winit::event::{ElementState, MouseButton};

use crate::resources::windowevents::{KeyboardEvent, MouseInputEvent, MouseMoveEvent};
use crate::scripting::scriptevent::ScriptEvent;

pub struct HandleInternalWindowEvents;

impl<'a> System<'a> for HandleInternalWindowEvents {
    type SystemData = (
        Write<'a, MouseInputEvent>,
        Write<'a, MouseMoveEvent>,
        Write<'a, KeyboardEvent>,
        Write<'a, CameraController>,
    );

    fn run(
        &mut self,
        (mut mouse_input_event, mut mouse_move_event, mut keyboard_event,mut cam_controller): Self::SystemData,
    ) {
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

pub struct HandleKeyboardEvent;
impl<'a> System<'a> for HandleKeyboardEvent {
    type SystemData = (
        Read<'a, KeyboardEvent>,
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(&mut self, (keyboard_event, callbacks, events, mut scripting_engine): Self::SystemData) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::OnKeyDown
                && !keyboard_event.handled
                && keyboard_event.info.state == ElementState::Pressed
            {
                let keyboard_event_args =
                    if let Some(virtual_keycode) = keyboard_event.info.virtual_keycode {
                        CallbackArgs::KeyboardEvent(virtual_keycode as u32)
                    } else {
                        CallbackArgs::KeyboardEvent(keyboard_event.info.scancode)
                    };
                callback.execute_with_args(&mut scripting_engine, keyboard_event_args);
            }
        }
    }
}
pub struct HandleMouseInputEvent;
impl<'a> System<'a> for HandleMouseInputEvent {
    type SystemData = (
        Read<'a, MouseInputEvent>,
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(&mut self, (mouse_event, callbacks, events, mut scripting_engine): Self::SystemData) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::OnMouseClick
                && !mouse_event.handled
                && mouse_event.info.1 == ElementState::Pressed
            {
                let mouse_button = match mouse_event.info.0 {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Other(other) => other,
                };
                callback.execute_with_args(
                    &mut scripting_engine,
                    CallbackArgs::MouseClickEvent(mouse_button),
                );
            }
        }
    }
}

pub struct HandleMouseMoveEvent;
impl<'a> System<'a> for HandleMouseMoveEvent {
    type SystemData = (
        Read<'a, MouseMoveEvent>,
        ReadStorage<'a, ScriptingCallback>,
        ReadStorage<'a, ScriptEvent>,
        WriteExpect<'a, HorizonScriptingEngine>,
    );

    fn run(
        &mut self,
        (mouse_move_event, callbacks, events, mut scripting_engine): Self::SystemData,
    ) {
        for (event, callback) in (&events, &callbacks).join() {
            if *event == ScriptEvent::OnMouseMove && !mouse_move_event.handled {
                callback.execute_with_args(
                    &mut scripting_engine,
                    CallbackArgs::MouseMoveEvent(mouse_move_event.info),
                );
            }
        }
    }
}
