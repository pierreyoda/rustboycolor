mod backend;
mod config;
mod emulator;
mod input;
mod logger;

use std::env;
use std::path::Path;
use std::error::Error;
extern crate time;
#[macro_use]
extern crate log;
extern crate getopts;
extern crate toml;

extern crate rustboylib;
use backend::{EmulatorBackend, sdl2};
use input::KeyboardBinding;

fn print_usage(opts: &getopts::Options) {
    let brief = concat!("rustboycolor : Game Boy (Color) emulator.\n\nUsage:\n",
                        "   rustboycolor [OPTIONS] ROM_FILE\n");
    println!("{}", opts.usage(&brief));
}

fn main() {
    // Logger initialization
    match logger::init_console_logger() {
        Err(error) => panic!(format!("Logging setup error : {}",
                                     error.description())),
        _          => (),
    }

    // Program options
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Print this help menu.");
    opts.optopt("k", "keyboard",
                "The keyboard configuration to use. QWERTY by default. \
                A custom binding can be defined in the configuration file.",
                "QWERTY/AZERTY");
    opts.optopt("c", "config",
                "The TOML configuration file to use. './config.toml' by default",
                "");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(why) => panic!(why.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&opts);
        return;
    }
    let config_file = match matches.opt_str("c") {
        Some(file) => file,
        None => "config.toml".to_string(),
    };
    let keyboard_binding = match matches.opt_str("k") {
        Some(binding) => match &binding[..] {
            "QWERTY" => KeyboardBinding::QWERTY,
            "AZERTY" => KeyboardBinding::AZERTY,
            _ => {
                warn!("Unkown keyboard binding '{}', reverting to QWERTY \
                      or configuration file.", binding);
                KeyboardBinding::FromConfigFile(config_file)
            }
        },
        _ => KeyboardBinding::FromConfigFile(config_file),
    };
    let rom = if !matches.free.is_empty() { matches.free[0].clone() }
        else { print_usage(&opts); return; };

    // Application launch
    let rom_path = Path::new(&rom);
    config::EmulatorAppConfig::new()
        .title("RustBoyColor - SDL 2")
        .width(800).height(600)
        .force_aspect(true)
        .keyboard_binding(keyboard_binding)
        .create_with_backend(Box::new(sdl2::BackendSDL2))
        .run(&rom_path);
}
