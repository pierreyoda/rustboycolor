use std::cmp;

use rustboylib::gpu::{SCREEN_W, SCREEN_H};
use super::backend::EmulatorBackend;
use super::emulator::EmulatorApplication;

// Macros to avoid boilerplate functions code.
macro_rules! config_set_param {
    ($setter_name: ident, $param_name: ident, $param_type: ty) => (
        #[allow(dead_code)]
        pub fn $setter_name(mut self, $param_name: $param_type)
            -> EmulatorAppConfig {
            self.$param_name = $param_name; self
        }
    )
}
macro_rules! config_get_param {
    ($getter_name: ident, $param_name: ident, $param_type: ty) => (
        pub fn $getter_name(&self) -> $param_type { self.$param_name.clone() }
    )
}

/// Enumerates the supported keyboard bindings for the virtual keypad.
/// TODO : add a Custom(...file?...) type, loaded from a file
#[derive(Clone, Debug)]
pub enum KeyboardBinding {
    QWERTY,
    AZERTY,
}

/// Structure facilitating the configuration and creation of the emulation
/// application.
#[derive(Clone, Debug)]
pub struct EmulatorAppConfig {
    /// The title of the emulator window.
    window_title        : &'static str,
    /// The desired width for the emulator display window.
    /// This is just a hint, the application may resize to reach a proper
    /// aspect ratio if the option 'window_force_aspect' is set to true.
    window_width        : u16,
    /// The desired height for the emulator display window.
    /// This is just a hint, the application may resize to reach a proper
    /// aspect ratio if the option 'window_force_aspect' is set to true.
    window_height       : u16,
    /// If true, the application will override the desired display size to
    /// respect the GB's aspect ratio.
    window_force_aspect : bool,
    /// The keyboard configuration. QWERTY by default.
    keyboard_binding    : KeyboardBinding,
}

impl EmulatorAppConfig {
    /// Return the display scale as (scale_h, scale_v) with the given
    /// configuration.
    pub fn compute_display_scale(&self) -> (u16, u16) {
        let scale_h = self.window_width  / (SCREEN_W as u16);
        let scale_v = self.window_height / (SCREEN_H as u16);
        if self.window_force_aspect {
            let min_scale = cmp::min(scale_h, scale_v);
            (min_scale, min_scale)
        }
        else {
            (scale_h, scale_v)
        }
    }

    /// Create and return a new 'EmulatorAppConfig' with the default values set.
    pub fn new() -> EmulatorAppConfig {
        EmulatorAppConfig {
            window_title: "RustBoyColor",
            window_width: SCREEN_W as u16 * 2,
            window_height: SCREEN_H as u16 * 2,
            window_force_aspect: true,
            keyboard_binding: KeyboardBinding::QWERTY,
        }
    }

    /// Create the 'EmulatorApplication' with this configuration and the
    /// given backend to use.
    pub fn create_with_backend<'a>(self, backend: Box<EmulatorBackend>)
        -> EmulatorApplication<'a> {
        EmulatorApplication::new(self, backend)
    }

    config_set_param!(title, window_title, &'static str);
    config_get_param!(get_title, window_title, &'static str);

    config_set_param!(width, window_width, u16);
    config_set_param!(height, window_height, u16);
    config_set_param!(force_aspect, window_force_aspect, bool);

    config_set_param!(keyboard_binding, keyboard_binding, KeyboardBinding);
    config_get_param!(get_keyboard_binding, keyboard_binding, KeyboardBinding);
}
