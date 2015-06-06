use std::path::Path;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use rustboylib;
use rustboylib::{cpu, mmu, mbc};
use super::backend::{EmulatorBackend, BackendMessage};
use super::config::EmulatorAppConfig;

/// Message emitted by the emulation loop to the UI backend.
pub enum EmulationMessage {
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
        // Communication channels
        let (tx_vm, rx_ui) = channel::<EmulationMessage>();
        let (tx_ui, rx_vm) = channel::<BackendMessage>();

        // VM loop, in a secondary thread
        let mbc: Box<mbc::MBC + Send> = match mbc::load_cartridge(&rom_path) {
            Ok(mbc)  => mbc,
            Err(why) => {
                error!("cannot load the cartridge : {}", why);
                return false;
            },
        };
        thread::spawn(move || {
            let mut mmu = mmu::MMU::new(mbc);
            let mut cpu = cpu::Cpu::<mmu::MMU>::new(mmu);
            emulation_loop(&mut cpu, tx_vm, rx_vm);
         });

        // UI loop, in the emulator's thread (should be the main thread)
        self.backend.run(self.config.clone(), tx_ui, rx_ui);
        true
    }
}

/// Emulation loop leveraging the rustboylib crate to emulate a Game Boy (Color).
fn emulation_loop(cpu: &cpu::Cpu<mmu::MMU>,
                  tx: Sender<EmulationMessage>, rx: Receiver<BackendMessage>) {
    use emulator::EmulationMessage::*;
    use backend::BackendMessage::*;

    info!("starting the emulation thread.");

    let mut running = true;

    'vm: loop {
        // Signals from the UI
        match rx.try_recv() {
            Ok(backend_message) => match backend_message {
                UpdateRunStatus(run) => running = run,
                KeyDown(key)         => {
                    println!("{:?}", key);
                },
                KeyUp(key)           => {
                    println!("{:?}", key);
                },
                Step                 => {},
                Reset                => {},
                Quit                 => {
                    running = false;
                    info!("terminating the virtual machine thread...");
                    tx.send(Finished).unwrap();
                    break 'vm;
                },
            },
            _                   => {},
        }
        println!("test");

        thread::sleep_ms(1000);
    }
}
