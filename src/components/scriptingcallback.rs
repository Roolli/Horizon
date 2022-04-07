#[cfg(not(target_arch = "wasm32"))]
use deno_core::v8;
#[cfg(not(target_arch = "wasm32"))]
use deno_core::v8::Handle;
use specs::*;
use winit::event::VirtualKeyCode;

// #[cfg(not(target_arch = "wasm32"))]
// use crate::scripting::scriptingengine::ScriptingEngineState;
// #[cfg(not(target_arch = "wasm32"))]
// use crate::V8ScriptingEngine;
use crate::scripting::util::horizonentity::HorizonEntity;
use crate::HorizonScriptingEngine;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: js_sys::Function,
}

#[cfg(target_arch = "wasm32")]
impl ScriptingCallback {
    pub fn new(callback: js_sys::Function) -> Self {
        Self { callback }
    }
    pub fn get_callback(&self) -> &js_sys::Function {
        &self.callback
    }
}
#[cfg(target_arch = "wasm32")]
impl<'a> ExecuteFunction<'a> for ScriptingCallback {
    type ScriptingEngine = (&'a mut HorizonScriptingEngine);
    fn execute_with_no_args(&self, scripting_engine: Self::ScriptingEngine) {
        self.get_callback().call0(&JsValue::UNDEFINED).unwrap();
    }

    fn execute_with_args(&self, scripting_engine: Self::ScriptingEngine, args: CallbackArgs) {
        if let CallbackArgs::Tick(dt) = args {
            self.get_callback()
                .call1(&JsValue::NULL, &JsValue::from(dt))
                .unwrap();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
#[storage(VecStorage)]
pub struct ScriptingCallback {
    callback: deno_core::v8::Global<deno_core::v8::Function>,
}
#[cfg(not(target_arch = "wasm32"))]
impl<'s> ExecuteFunction<'s> for ScriptingCallback {
    type ScriptingEngine = (&'s mut HorizonScriptingEngine);
    fn execute_with_no_args(&self, additional_args: Self::ScriptingEngine) {
        let engine = additional_args;
        let js = &mut engine.js_runtime;
        let scope = &mut js.handle_scope();
        let recv = v8::Integer::new(scope, 1).into();
        self.callback.open(scope).call(scope, recv, &[]);
    }

    fn execute_with_args(&self, additional_args: Self::ScriptingEngine, args: CallbackArgs) {
        let engine = additional_args;
        let js = &mut engine.js_runtime;
        let scope = &mut js.handle_scope();
        let recv = v8::Integer::new(scope, 1).into();
        match args {
            CallbackArgs::Tick(t) => {
                let val = v8::Number::new(scope, t as f64).into();
                self.callback.open(scope).call(scope, recv, &[val]);
            }
            CallbackArgs::KeyboardEvent(keycode) => {
                let val = v8::Integer::new(scope, keycode as i32).into();
                self.callback.open(scope).call(scope, recv, &[val]);
            }
            CallbackArgs::MouseMoveEvent((rel_x, rel_y)) => {
                let x = v8::Number::new(scope, rel_x).into();
                let y = v8::Number::new(scope, rel_y).into();
                self.callback.open(scope).call(scope, recv, &[x, y]);
            }
            CallbackArgs::MouseClickEvent(button_id) => {
                let num = v8::Integer::new(scope, button_id as i32).into();
                self.callback.open(scope).call(scope, recv, &[num]);
            }
            CallbackArgs::EntityCollision(entity_one, entity_two) => {
                let entity_one = deno_core::serde_v8::to_v8(scope, entity_one).unwrap();
                let entity_two = deno_core::serde_v8::to_v8(scope, entity_two).unwrap();
                self.callback
                    .open(scope)
                    .call(scope, recv, &[entity_one, entity_two]);
            }
            CallbackArgs::None => {
                self.callback.open(scope).call(scope, recv, &[]);
            }
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl ScriptingCallback {
    pub fn new(callback: deno_core::v8::Global<deno_core::v8::Function>) -> Self {
        Self { callback }
    }
    pub fn get_callback(&self) -> &deno_core::v8::Global<deno_core::v8::Function> {
        &self.callback
    }
}

pub trait ExecuteFunction<'s> {
    type ScriptingEngine;
    fn execute_with_no_args(&self, additional_args: Self::ScriptingEngine);
    fn execute_with_args(&self, additional_args: Self::ScriptingEngine, args: CallbackArgs);
}
#[derive(Debug)]
pub enum CallbackArgs {
    None,
    Tick(f32),
    KeyboardEvent(u32),
    MouseClickEvent(u16),
    MouseMoveEvent((f64, f64)),
    EntityCollision(HorizonEntity, HorizonEntity),
}
impl CallbackArgs {
    pub fn from_winit_keycode_to_js(keycode: VirtualKeyCode) -> CallbackArgs {
        CallbackArgs::KeyboardEvent(match keycode {
            VirtualKeyCode::Key1 => 49,
            VirtualKeyCode::Key2 => 50,
            VirtualKeyCode::Key3 => 51,
            VirtualKeyCode::Key4 => 52,
            VirtualKeyCode::Key5 => 53,
            VirtualKeyCode::Key6 => 54,
            VirtualKeyCode::Key7 => 55,
            VirtualKeyCode::Key8 => 56,
            VirtualKeyCode::Key9 => 57,
            VirtualKeyCode::Key0 => 48,
            VirtualKeyCode::A => 65,
            VirtualKeyCode::B => 66,
            VirtualKeyCode::C => 67,
            VirtualKeyCode::D => 68,
            VirtualKeyCode::E => 69,
            VirtualKeyCode::F => 70,
            VirtualKeyCode::G => 71,
            VirtualKeyCode::H => 72,
            VirtualKeyCode::I => 73,
            VirtualKeyCode::J => 74,
            VirtualKeyCode::K => 75,
            VirtualKeyCode::L => 76,
            VirtualKeyCode::M => 77,
            VirtualKeyCode::N => 78,
            VirtualKeyCode::O => 79,
            VirtualKeyCode::P => 80,
            VirtualKeyCode::Q => 81,
            VirtualKeyCode::R => 82,
            VirtualKeyCode::S => 83,
            VirtualKeyCode::T => 84,
            VirtualKeyCode::U => 85,
            VirtualKeyCode::V => 86,
            VirtualKeyCode::W => 87,
            VirtualKeyCode::X => 88,
            VirtualKeyCode::Y => 89,
            VirtualKeyCode::Z => 90,
            VirtualKeyCode::Escape => 27,
            VirtualKeyCode::F1 => 112,
            VirtualKeyCode::F2 => 113,
            VirtualKeyCode::F3 => 114,
            VirtualKeyCode::F4 => 115,
            VirtualKeyCode::F5 => 116,
            VirtualKeyCode::F6 => 117,
            VirtualKeyCode::F7 => 118,
            VirtualKeyCode::F8 => 119,
            VirtualKeyCode::F9 => 120,
            VirtualKeyCode::F10 => 121,
            VirtualKeyCode::F11 => 122,
            VirtualKeyCode::F12 => 123,
            VirtualKeyCode::F13 => 0,
            VirtualKeyCode::F14 => 0,
            VirtualKeyCode::F15 => 0,
            VirtualKeyCode::F16 => 0,
            VirtualKeyCode::F17 => 0,
            VirtualKeyCode::F18 => 0,
            VirtualKeyCode::F19 => 0,
            VirtualKeyCode::F20 => 0,
            VirtualKeyCode::F21 => 0,
            VirtualKeyCode::F22 => 0,
            VirtualKeyCode::F23 => 0,
            VirtualKeyCode::F24 => 0,
            VirtualKeyCode::Snapshot => 0,
            VirtualKeyCode::Scroll => 0,
            VirtualKeyCode::Pause => 19,
            VirtualKeyCode::Insert => 45,
            VirtualKeyCode::Home => 36,
            VirtualKeyCode::Delete => 46,
            VirtualKeyCode::End => 35,
            VirtualKeyCode::PageDown => 34,
            VirtualKeyCode::PageUp => 35,
            VirtualKeyCode::Left => 37,
            VirtualKeyCode::Up => 38,
            VirtualKeyCode::Right => 39,
            VirtualKeyCode::Down => 40,
            VirtualKeyCode::Back => 8,
            VirtualKeyCode::Return => 13,
            VirtualKeyCode::Space => 32,
            VirtualKeyCode::Compose => 0,
            VirtualKeyCode::Caret => 0,
            VirtualKeyCode::Numlock => 144,
            VirtualKeyCode::Numpad0 => 96,
            VirtualKeyCode::Numpad1 => 97,
            VirtualKeyCode::Numpad2 => 98,
            VirtualKeyCode::Numpad3 => 99,
            VirtualKeyCode::Numpad4 => 100,
            VirtualKeyCode::Numpad5 => 101,
            VirtualKeyCode::Numpad6 => 102,
            VirtualKeyCode::Numpad7 => 103,
            VirtualKeyCode::Numpad8 => 104,
            VirtualKeyCode::Numpad9 => 105,
            VirtualKeyCode::NumpadAdd => 107,
            VirtualKeyCode::NumpadDivide => 111,
            VirtualKeyCode::NumpadDecimal => 110,
            VirtualKeyCode::NumpadComma => 188,
            VirtualKeyCode::NumpadEnter => 0,
            VirtualKeyCode::NumpadEquals => 187,
            VirtualKeyCode::NumpadMultiply => 106,
            VirtualKeyCode::NumpadSubtract => 109,
            VirtualKeyCode::AbntC1 => 0,
            VirtualKeyCode::AbntC2 => 0,
            VirtualKeyCode::Apostrophe => 222,
            VirtualKeyCode::Apps => 0,
            VirtualKeyCode::Asterisk => 0,
            VirtualKeyCode::At => 0,
            VirtualKeyCode::Ax => 0,
            VirtualKeyCode::Backslash => 220,
            VirtualKeyCode::Calculator => 0,
            VirtualKeyCode::Capital => 0,
            VirtualKeyCode::Colon => 0,
            VirtualKeyCode::Comma => 188,
            VirtualKeyCode::Convert => 0,
            VirtualKeyCode::Equals => 187,
            VirtualKeyCode::Grave => 0,
            VirtualKeyCode::Kana => 0,
            VirtualKeyCode::Kanji => 0,
            VirtualKeyCode::LAlt => 18,
            VirtualKeyCode::LBracket => 0,
            VirtualKeyCode::LControl => 17,
            VirtualKeyCode::LShift => 16,
            VirtualKeyCode::LWin => 0,
            VirtualKeyCode::Mail => 0,
            VirtualKeyCode::MediaSelect => 0,
            VirtualKeyCode::MediaStop => 0,
            VirtualKeyCode::Minus => 0,
            VirtualKeyCode::Mute => 0,
            VirtualKeyCode::MyComputer => 0,
            VirtualKeyCode::NavigateForward => 0,
            VirtualKeyCode::NavigateBackward => 0,
            VirtualKeyCode::NextTrack => 0,
            VirtualKeyCode::NoConvert => 0,
            VirtualKeyCode::OEM102 => 0,
            VirtualKeyCode::Period => 0,
            VirtualKeyCode::PlayPause => 0,
            VirtualKeyCode::Plus => 0,
            VirtualKeyCode::Power => 0,
            VirtualKeyCode::PrevTrack => 0,
            VirtualKeyCode::RAlt => 18,
            VirtualKeyCode::RBracket => 0,
            VirtualKeyCode::RControl => 17,
            VirtualKeyCode::RShift => 16,
            VirtualKeyCode::RWin => 0,
            VirtualKeyCode::Semicolon => 186,
            VirtualKeyCode::Slash => 191,
            VirtualKeyCode::Sleep => 0,
            VirtualKeyCode::Stop => 0,
            VirtualKeyCode::Sysrq => 0,
            VirtualKeyCode::Tab => 9,
            VirtualKeyCode::Underline => 0,
            VirtualKeyCode::Unlabeled => 0,
            VirtualKeyCode::VolumeDown => 0,
            VirtualKeyCode::VolumeUp => 0,
            VirtualKeyCode::Wake => 0,
            VirtualKeyCode::WebBack => 0,
            VirtualKeyCode::WebFavorites => 0,
            VirtualKeyCode::WebForward => 0,
            VirtualKeyCode::WebHome => 0,
            VirtualKeyCode::WebRefresh => 0,
            VirtualKeyCode::WebSearch => 0,
            VirtualKeyCode::WebStop => 0,
            VirtualKeyCode::Yen => 0,
            VirtualKeyCode::Copy => 0,
            VirtualKeyCode::Paste => 0,
            VirtualKeyCode::Cut => 0,
        })
    }
}
