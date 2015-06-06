use std::path::Path;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use std::thread;

use rustboylib;
use super::backend::EmulatorBackend;
use super::config::EmulatorAppConfig;

/// Message emitted by the emulation loop to the UI backend.
pub enum EmulatorMessage {
    /// Signal that the emulation is finished, emitted either after a
    /// 'BackendMessage::Quit' signal was received or when the virtual machine
    /// finished the execution of its cartridge.
    Finished,
}

/// The backend-agnostic RustBoyColor emulator application.
/// Communication between the virtual machine's emulation loop and the
/// backend's UI loop is done with 2 channels using respectively
/// 'Chip8VMCommand' and 'Chip8UICommand'.
pub struct EmulatorApplication<'a> {
    /// The application configuration.
    config  : EmulatorAppConfig,
    /// Pointer to the heap-allocated backend responsible for running the
    /// actual UI loop in the main thread.
    backend : Box<EmulatorBackend + 'a>,
}

impl<'a> EmulatorApplication<'a> {
    pub fn new(config: EmulatorAppConfig, backend: Box<EmulatorBackend>)
        -> EmulatorApplication<'a> {
        EmulatorApplication { config: config, backend: backend }
    }

    /// Run the emulator application with the given cartridge.
    /// Return true if all went well, false otherwise.
    /// TODO : more flexible run function (maybe a LoadRomCommand ?)
    pub fn run(&mut self, rom_path: &Path) -> bool {
        // Cartrige loading
        info!("loading the ROM file \"{}\"...", rom_path.display());
        let mbc = match rustboylib::mbc::load_cartridge(rom_path) {
            Ok(mbc)  => mbc,
            Err(why) => {
                error!("cannot load the cartridge : {}", why);
                return false;
            }
        };
        info!("successfully loaded the cartridge");

        true
    }
}

/// Emulation loop leveraging the rustboylib crate to emulate a Game Boy (Color).
fn emulation_loop() {
    info!("starting the emulation thread.");
    'vm : loop {
        println!("emu : ran once", );
        break 'vm;
    }
}
