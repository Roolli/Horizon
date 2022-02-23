pub struct ResizeEvent {
    pub new_size: winit::dpi::PhysicalSize<u32>,
    pub scale_factor: Option<f64>,
    pub handled: bool,
}

pub struct MouseInputEvent {
    pub info: (winit::event::MouseButton, winit::event::ElementState),
    pub handled: bool,
}
impl Default for MouseInputEvent {
    fn default() -> Self {
        Self {
            info: (
                winit::event::MouseButton::Left,
                winit::event::ElementState::Pressed,
            ),
            handled: true,
        }
    }
}
pub struct KeyboardEvent {
    pub info: winit::event::KeyboardInput,
    pub handled: bool,
}

impl Default for KeyboardEvent {
    fn default() -> Self {
        Self {
            info: winit::event::KeyboardInput {
                virtual_keycode: None,
                state: winit::event::ElementState::Pressed,
                scancode: 0,
                modifiers: winit::event::ModifiersState::ALT,
            },
            handled: true,
        }
    }
}
pub struct MouseMoveEvent {
    pub info: (f64, f64),
    pub handled: bool,
}
impl Default for MouseMoveEvent {
    fn default() -> Self {
        Self {
            info: (0.0, 0.0),
            handled: true,
        }
    }
}
