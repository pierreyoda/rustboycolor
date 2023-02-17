mod backend;
mod config;
mod emulator;
mod input;
mod logger;

use std::path::{Path, PathBuf};

#[macro_use]
extern crate log;

use clap::{Arg, Parser, ValueEnum};

use crate::backend::sdl2;
use crate::input::KeyboardBinding;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum KeyboardMode {
    Azerty,
    Qwerty,
}

#[derive(Debug, Parser)]
#[clap(author, version = "alpha", about = "Game Boy (Color) emulator", long_about = None)]
struct Args {
    #[clap(help = "The ROM file to play.")]
    rom_file: PathBuf,

    #[clap(
        short,
        long,
        value_name = "CONFIG_FILE",
        help = "Sets the configuration file to use. './config.toml' by default."
    )]
    config_file: Option<PathBuf>,

    #[clap(
        short,
        long,
        value_enum,
        value_name = "keyboard",
        help = "Sets the keyboard configuration to use. QWERTY by default. A custom binding can be defined in the configuration file."
    )]
    keyboard_mode: Option<KeyboardMode>,
}

fn app_options_from_args(args: &Args) -> config::EmulatorAppConfig {
    let config_file = args.config_file.clone().unwrap_or("config.toml".into());
    let keyboard_binding = match args.keyboard_mode {
        Some(mode) => match mode {
            KeyboardMode::Qwerty => KeyboardBinding::QWERTY,
            KeyboardMode::Azerty => KeyboardBinding::AZERTY,
        },
        _ => KeyboardBinding::FromConfigFile(config_file.clone()),
    };

    let config = match config::EmulatorAppConfig::from_file(&config_file) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "cannot use the configuration file \"{}\": {}",
                config_file.display(),
                e
            );
            config::EmulatorAppConfig::new()
        }
    };
    config.keyboard_binding(keyboard_binding)
}

fn main() {
    // Logger initialization
    if let Err(error) = logger::init_console_logger() {
        panic!("Logging setup error : {}", error);
    }

    // CLI options
    let args: Args = Args::parse();
    let rom = args.rom_file.clone();

    // Application launch
    app_options_from_args(&args)
        .title("RustBoyColor - SDL2")
        .create_with_backend(Box::new(sdl2::BackendSDL2))
        .run(Path::new(&rom), true);
}
