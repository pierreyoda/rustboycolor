use std::error::Error;
#[macro_use]
extern crate log;

extern crate rustboylib;

mod logger;

fn main() {
    // Logger initialization
    match logger::init_console_logger() {
        Err(error) => panic!(format!("Logging setup error : {}",
                                     error.description())),
        _          => (),
    }
}
