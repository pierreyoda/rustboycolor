use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::backend::{BackendMessage, EmulatorBackend};
use crate::config::EmulatorAppConfig;
use rustboylib::cpu::{CycleType, CPU_CLOCK_SPEED};
use rustboylib::gpu::RGB;
use rustboylib::{cpu, mbc, mmu};

/// Message emitted by the emulation loop to the UI backend.
pub enum EmulationMessage {
    /// Update the display.
    UpdateDisplay(Vec<RGB>),
    /// Signal that the emulation is finished, emitted either after a
    /// 'BackendMessage::Quit' signal was received or when the virtual machine
    /// finished the execution of its cartridge.
    Finished,
}

/// The backend-agnostic RustBoyColor emulator application.
///
/// Communication between the virtual machine's emulation loop and the
/// backend's UI loop is done with 2 channels using respectively
/// 'Chip8VMCommand' and 'Chip8UICommand'.
pub struct EmulatorApplication {
    /// The application configuration.
    config: EmulatorAppConfig,
    /// Pointer to the heap-allocated backend responsible for running the
    /// actual UI loop in the main thread.
    backend: Box<dyn EmulatorBackend>,
}

impl EmulatorApplication {
    pub fn new(
        config: EmulatorAppConfig,
        backend: Box<dyn EmulatorBackend>,
    ) -> EmulatorApplication {
        EmulatorApplication { config, backend }
    }

    /// Run the emulator application with the given cartridge.
    /// Return true if all went well, false otherwise.
    /// TODO: more flexible run function (maybe a LoadRomCommand ?)
    pub fn run(&mut self, rom_path: &Path, skip_bios: bool) -> bool {
        // Communication channels
        let (tx_vm, rx_ui) = channel::<EmulationMessage>();
        let (tx_ui, rx_vm) = channel::<BackendMessage>();

        // VM loop, in a secondary thread
        let mbc: Box<dyn mbc::MBC + Send> = match mbc::load_cartridge(&rom_path) {
            Ok(mbc) => mbc,
            Err(why) => {
                error!("cannot load the cartridge : {}", why);
                return false;
            }
        };
        match thread::Builder::new()
            .name("rustboylib_vm".into())
            .spawn(move || {
                let mmu = mmu::MMU::new(mbc, false, skip_bios, None);
                let mut cpu = cpu::Cpu::<mmu::MMU>::new(mmu);
                if skip_bios {
                    cpu.post_bios();
                }
                emulation_loop(&mut cpu, tx_vm, rx_vm);
            }) {
            Err(why) => {
                error!("cannot spawn the VM thread: {}", why);
                return false;
            }
            _ => {}
        };

        // UI loop, in the emulator's thread (should be the main thread)
        self.backend.run(self.config.clone(), tx_ui, rx_ui);

        true
    }
}

/// Emulation loop leveraging the rustboylib crate to emulate a Game Boy (Color).
fn emulation_loop(
    cpu: &mut cpu::Cpu<mmu::MMU>,
    tx: Sender<EmulationMessage>,
    rx: Receiver<BackendMessage>,
) {
    use crate::backend::BackendMessage::*;
    use crate::emulator::EmulationMessage::*;

    info!("starting the emulation thread.");

    let mut running = true;
    // target CPU clock cycles per second
    let frame_ticks = (CPU_CLOCK_SPEED / 1000 * 16) as CycleType;
    let mut ticks: CycleType = 0;

    'vm: loop {
        // Signals from the UI
        match rx.try_recv() {
            Ok(backend_message) => match backend_message {
                UpdateRunStatus(run) => running = run,
                KeyDown(key) => cpu.mem.key_down(&key),
                KeyUp(key) => cpu.mem.key_up(&key),
                Step => {}
                Reset => {}
                Quit => {
                    running = false;
                    info!("terminating the emulation thread...");
                    tx.send(Finished).unwrap();
                    break 'vm;
                }
            },
            _ => {}
        }

        if !running {
            continue;
        }

        while ticks < frame_ticks {
            ticks += cpu.step();
        }
        ticks -= frame_ticks;
        if let Some(frame_buffer) = cpu.mem.frame_buffer() {
            tx.send(UpdateDisplay(frame_buffer)).unwrap();
        }

        // thread::sleep(Duration::from_millis(1));
    }
}
