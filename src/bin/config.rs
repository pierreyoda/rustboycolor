use std::cmp;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml;

use super::backend::EmulatorBackend;
use super::emulator::EmulatorApplication;
use super::input::KeyboardBinding;
use rustboylib::gpu::{SCREEN_H, SCREEN_W};

// Default display scale, i.e. the actual size (in pixels) of each individual GameBoy pixel.
const DEFAULT_SCALE: u16 = 2;

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

/// Structure facilitating the configuration and creation of the emulation
/// application.
#[derive(Clone, Debug)]
pub struct EmulatorAppConfig {
    /// The title of the emulator window.
    window_title: &'static str,
    /// The desired width for the emulator display window.
    /// This is just a hint, the application may resize to reach a proper
    /// aspect ratio if the option 'window_force_aspect' is set to true.
    window_width: u16,
    /// The desired height for the emulator display window.
    /// This is just a hint, the application may resize to reach a proper
    /// aspect ratio if the option 'window_force_aspect' is set to true.
    window_height: u16,
    /// If true, the application will override the desired display size to
    /// respect the GB's aspect ratio. True by default.
    window_force_aspect: bool,
    /// The keyboard configuration. QWERTY by default.
    keyboard_binding: KeyboardBinding,
    /// Display the FPS count.
    display_fps: bool,
}

impl EmulatorAppConfig {
    /// Return the display scale as (scale_h, scale_v) with the given
    /// configuration.
    pub fn compute_display_scale(&self) -> (u16, u16) {
        let scale_h = self.window_width / (SCREEN_W as u16);
        let scale_v = self.window_height / (SCREEN_H as u16);
        if self.window_force_aspect {
            let min_scale = cmp::min(scale_h, scale_v);
            (min_scale, min_scale)
        } else {
            (scale_h, scale_v)
        }
    }

    /// Create and return a new 'EmulatorAppConfig' with the default values set.
    pub fn new() -> EmulatorAppConfig {
        EmulatorAppConfig {
            window_title: "RustBoyColor",
            window_width: SCREEN_W as u16 * DEFAULT_SCALE,
            window_height: SCREEN_H as u16 * DEFAULT_SCALE,
            window_force_aspect: true,
            keyboard_binding: KeyboardBinding::QWERTY,
            display_fps: false,
        }
    }

    /// Create and return a new 'EmulatorAppConfig' with the default values set,
    /// and with all valid properties from the given configuration file set.
    pub fn from_file(filepath: &str) -> Result<EmulatorAppConfig, String> {
        let mut config = EmulatorAppConfig::new();

        let file_path = Path::new(filepath);
        let mut file_content = String::new();
        r#try!(File::open(file_path)
            .and_then(|mut f| f.read_to_string(&mut file_content))
            .map_err(|_| format!("could not load the config file : {}", file_path.display())));

        let table_value = match file_content.parse::<toml::Value>() {
            Ok(value) => value,
            Err(err) => {
                return Err(format!(
                    "parsing error in config file \"{}\" : {}",
                    file_path.display(),
                    err
                ))
            }
        };
        let table = table_value.as_table().unwrap();

        info!(
            "reading configuration from file \"{}\"...",
            file_path.display()
        );
        if let Some(value) = table.get("display") {
            let display = value
                .as_table()
                .expect("config file error : no display section");
            match lookup_bool_value("force_aspect", display) {
                Ok(force_aspect) => config.window_force_aspect = force_aspect,
                Err(error) => warn!("{}", error),
            }
            match lookup_bool_value("show_fps", display) {
                Ok(show_fps) => config.display_fps = show_fps,
                Err(error) => warn!("{}", error),
            }
            match lookup_int_value("width", display) {
                Ok(width) => {
                    if width > 0 && width as u16 >= SCREEN_W as u16 {
                        config.window_width = width as u16;
                    } else {
                        warn!("invalid display width");
                    }
                }
                Err(error) => warn!("{}", error),
            }
            match lookup_int_value("height", display) {
                Ok(height) => {
                    if height > 0 && height as u16 >= SCREEN_H as u16 {
                        config.window_height = height as u16;
                    } else {
                        warn!("invalid display height");
                    }
                }
                Err(error) => warn!("{}", error),
            }
        } else {
            warn!("no display section in config file");
        }

        info!("configuration reading done.");

        Ok(config)
    }

    /// Create the 'EmulatorApplication' with this configuration and the
    /// given backend to use.
    pub fn create_with_backend<'a>(
        self,
        backend: Box<dyn EmulatorBackend>,
    ) -> EmulatorApplication<'a> {
        EmulatorApplication::new(self, backend)
    }

    config_set_param!(title, window_title, &'static str);
    config_get_param!(get_title, window_title, &'static str);

    config_set_param!(width, window_width, u16);
    config_set_param!(height, window_height, u16);
    config_set_param!(force_aspect, window_force_aspect, bool);

    config_set_param!(keyboard_binding, keyboard_binding, KeyboardBinding);
    config_get_param!(get_keyboard_binding, keyboard_binding, KeyboardBinding);

    config_set_param!(display_fps, display_fps, bool);
    config_get_param!(get_display_fps, display_fps, bool);
}

fn lookup_bool_value(key: &'static str, table: &toml::value::Table) -> Result<bool, String> {
    if let Some(value) = table.get(key) {
        match *value {
            toml::Value::Boolean(boolean) => Ok(boolean),
            _ => {
                return Err(format!(
                    "config::lookup_bool_value : key '{}' does not correspond to \
                     a boolean",
                    key
                ))
            }
        }
    } else {
        Err(format!(
            "config::lookup_bool_value : key '{}' was not found in the given table",
            key
        ))
    }
}

fn lookup_int_value(key: &'static str, table: &toml::value::Table) -> Result<i64, String> {
    if let Some(value) = table.get(key) {
        match *value {
            toml::Value::Integer(int) => Ok(int),
            _ => {
                return Err(format!(
                    "config::lookup_int_value : key '{}' does not correspond to \
                     an integer",
                    key
                ))
            }
        }
    } else {
        Err(format!(
            "config::lookup_int_value : key '{}' was not found in the given table",
            key
        ))
    }
}
