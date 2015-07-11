pub mod sdl2;

use std::sync::mpsc::{Sender, Receiver};

use rustboylib::keypad::KeypadKey;
use super::config::EmulatorAppConfig;
use super::emulator::EmulationMessage;
use super::input::KeyboardBinding;

/// Message emitted by the backend UI loop to the emulation core.
pub enum BackendMessage {
    /// Set the emulation state (running for true, paused for false).
    UpdateRunStatus(bool),
    /// Notify that a key was pressed.
    KeyDown(KeypadKey),
    /// Notify that a key was released.
    KeyUp(KeypadKey),
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
    fn run(&mut self, config: EmulatorAppConfig,
           tx: Sender<BackendMessage>, rx: Receiver<EmulationMessage>);
}
