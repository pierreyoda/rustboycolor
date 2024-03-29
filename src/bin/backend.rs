pub mod sdl2;

use std::sync::mpsc::{Receiver, Sender};

use super::config::EmulatorAppConfig;
use super::emulator::EmulationMessage;
use rustboylib::joypad::JoypadKey;

/// Message emitted by the backend UI loop to the emulation core.
pub enum BackendMessage {
    /// Set the emulation state (running if true, paused if false).
    UpdateRunStatus(bool),
    /// Notify that a key was pressed.
    KeyDown(JoypadKey),
    /// Notify that a key was released.
    KeyUp(JoypadKey),
    /// When the emulation is paused, perform a single step.
    Step,
    /// Reset the emulation.
    Reset,
    /// Signal to gracefully shutdown the virtual machine. The backend
    /// must then await for confirmation from the virtual machine.
    Quit,
}

/// Trait that any emulator backend must implement.
pub trait EmulatorBackend {
    /// Launch and run the UI loop with the given configuration.
    fn run(
        &mut self,
        config: EmulatorAppConfig,
        tx: Sender<BackendMessage>,
        rx: Receiver<EmulationMessage>,
    );
}
