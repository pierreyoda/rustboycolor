mod backend;
mod config;
mod emulator;
mod logger;

use std::env;
use std::path::Path;
use std::error::Error;
extern crate time;
#[macro_use]
extern crate log;
extern crate getopts;

extern crate rustboylib;
use backend::{EmulatorBackend, sdl2};

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
                "The keyboard configuration to use. QWERTY by default.",
                "QWERTY/AZERTY");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(why) => panic!(why.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&opts);
        return;
    }
    let rom = if !matches.free.is_empty() { matches.free[0].clone() }
        else { print_usage(&opts); return; };

    // Application launch
    let rom_path = Path::new(&rom);
    config::EmulatorAppConfig::new()
        .title("RustBoyColor - SDL 2")
        .width(800).height(600)
        .force_aspect(true)
        .create_with_backend(Box::new(sdl2::BackendSDL2))
        .run(&rom_path);
}
