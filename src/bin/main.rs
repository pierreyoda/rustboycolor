mod backend;
mod config;
mod emulator;
mod input;
mod logger;

use std::path::Path;
use std::error::Error;

#[macro_use]
extern crate log;
extern crate clap;
extern crate toml;

use clap::{App, Arg, ArgMatches};

extern crate rustboylib;
use backend::sdl2;
use input::KeyboardBinding;

fn app_options_from_matches(matches: &ArgMatches) -> config::EmulatorAppConfig {
    let config_file = matches.value_of("config").unwrap_or("config.toml");
    let keyboard_binding = match matches.value_of("keyboard") {
        Some(binding) => {
            match &binding[..] {
                "QWERTY" => KeyboardBinding::QWERTY,
                "AZERTY" => KeyboardBinding::AZERTY,
                _ => {
                    warn!("Unkown keyboard binding '{}', reverting to QWERTY or configuration \
                           file.",
                          binding);
                    KeyboardBinding::FromConfigFile(config_file.into())
                }
            }
        }
        _ => KeyboardBinding::FromConfigFile(config_file.into()),
    };

    let config = match config::EmulatorAppConfig::from_file(&config_file) {
        Ok(c) => c,
        Err(e) => {
            warn!("cannot use the configuration file \"{}\": {}",
                  config_file,
                  e);
            config::EmulatorAppConfig::new()
        }
    };
    config.keyboard_binding(keyboard_binding)
}

fn main() {
    // Logger initialization
    if let Err(error) = logger::init_console_logger() {
        panic!(format!("Logging setup error : {}", error.description()));
    }

    // Program options
    let matches = App::new("rustboycolor")
        .version("alpha")
        .about("Game Boy (Color) emulator")
        .arg(Arg::with_name("keyboard")
            .help("Sets the keyboard configuration to use. QWERTY by default. \
                   A custom binding can be defined in the configuration file.")
            .short("k")
            .long("keyboard")
            .value_name("QWERTY/AZERTY")
            .takes_value(true))
        .arg(Arg::with_name("config")
            .help("Sets the configuration file to use. './config.toml' by default.")
            .short("c")
            .long("config")
            .value_name("CONFIG_FILE")
            .takes_value(true))
        .arg(Arg::with_name("ROM_FILE")
            .help("The ROM file to play.")
            .required(true)
            .index(1))
        .get_matches();

    let rom = matches.value_of("ROM_FILE").unwrap();

    // Application launch
    let rom_path = Path::new(&rom);
    app_options_from_matches(&matches)
        .title("RustBoyColor - SDL2")
        .create_with_backend(Box::new(sdl2::BackendSDL2))
        .run(&rom_path, true);
}
