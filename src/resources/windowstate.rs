use std::collections::HashSet;

//TODO: modify window state in the internal window event handler
pub struct WindowState {
    pub cursor_state: bool,
    pub pressed_keys: HashSet<u32>,
    pub mouse_location: (f32, f32),
}
