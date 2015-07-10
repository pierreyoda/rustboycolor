use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
extern crate sdl2;
use self::sdl2::event::Event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::rect::Rect;
use self::sdl2::pixels::Color;

use rustboylib::gpu::{SCREEN_W, SCREEN_H};
use rustboylib::keypad::KeypadKey;
use super::{EmulatorBackend, BackendMessage};
use config::{EmulatorAppConfig, KeyboardBinding};
use emulator::EmulationMessage;

/// The SDL 2 backend, using rust-sdl2.
pub struct BackendSDL2;

impl EmulatorBackend for BackendSDL2 {
    fn run(&mut self, config: EmulatorAppConfig,
       tx: Sender<BackendMessage>, rx: Receiver<EmulationMessage>) {
        use emulator::EmulationMessage::*;
        use backend::BackendMessage::*;

        info!("starting the main application thread.");
        let (scale_h, scale_v) = config.compute_display_scale();
        let w = (SCREEN_W as u32) * (scale_h as u32);
        let h = (SCREEN_H as u32) * (scale_v as u32);
        info!("display scale = ({}, {}).", scale_h, scale_v);

        // SDL 2 initialization
        let mut context = sdl2::init().video().build().unwrap();
        let window = match context.window(config.get_title(), w, h)
            .position_centered().opengl().build() {
            Ok(window) => window,
            Err(why)   => {
                error!("SDL2 backend failed to create the window : {}", why);
                return;
            },
        };
        let mut renderer = match window.renderer().build() {
            Ok(renderer) => renderer,
            Err(why)     => {
                error!("SDL2 backend failed to create the renderer : {}", why);
                return;
            },
        };
        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.present();
        let mut events = context.event_pump();
        let key_binds = get_key_bindings(&config.get_keyboard_binding());

        // is the emulation paused ?
        let mut paused = false;
        // avoid spamming 'Event::KeyDown' events for the same key
        let mut last_key: Option<Keycode> = None;

        // Main loop
        'ui: loop {
            // Event loop
            for event in events.poll_iter() {
                match event {
                    Event::Quit {..} => { paused = true; tx.send(Quit).unwrap(); },
                    Event::KeyDown { keycode: Some(keycode), ..} => {
                        if !last_key.is_none() && keycode == last_key.unwrap() {
                            continue;
                        }
                        match keycode {
                            // quit
                            Keycode::Escape => {
                                paused = true; tx.send(Quit).unwrap();
                            },
                            // toggle pause
                            Keycode::Return => {
                                tx.send(UpdateRunStatus(paused)).unwrap();
                                paused = !paused;
                            },
                            _ => if !paused {
                                match key_binds.get(&keycode) {
                                    Some(keypad_key) => {
                                        tx.send(KeyDown(*keypad_key)).unwrap();
                                    },
                                    _                => {},
                                }
                            }
                        }
                        last_key = Some(keycode);
                    },
                    Event::KeyUp { keycode: Some(keycode), ..} if !paused => {
                        match key_binds.get(&keycode) {
                            Some(keypad_key) => {
                                tx.send(KeyUp(*keypad_key)).unwrap();
                            },
                            _                => {},
                        }
                        if !last_key.is_none() && keycode == last_key.unwrap() {
                            last_key = None;
                        }
                    }
                    _ => continue,
                }
            }

            // Signals from the VM
            match rx.try_recv() {
                Ok(emulation_message) => match emulation_message {
                    Finished => break 'ui,
                },
                _                     => {},
            }
        }

        info!("terminating the main application thread.")
    }
}

/// Return the 'HashMap<Keycode, KeypadKey>' translating between SDL 2 code keys
/// and rustboylib's keypad keys, according to the given keyboard configuration.
fn get_key_bindings(binding: &KeyboardBinding) -> HashMap<Keycode, KeypadKey> {
    let mut hm = HashMap::new();

    hm.insert(Keycode::S, KeypadKey::Down);
    hm.insert(Keycode::D, KeypadKey::Right);
    hm.insert(Keycode::G, KeypadKey::B);
    hm.insert(Keycode::Y, KeypadKey::A);
    hm.insert(Keycode::C, KeypadKey::Start);

    match *binding {
        KeyboardBinding::QWERTY => {
            hm.insert(Keycode::W, KeypadKey::Up);
            hm.insert(Keycode::A, KeypadKey::Left);
            hm.insert(Keycode::Z, KeypadKey::Select);
        },
        KeyboardBinding::AZERTY => {
            hm.insert(Keycode::Z, KeypadKey::Up);
            hm.insert(Keycode::Q, KeypadKey::Left);
            hm.insert(Keycode::W, KeypadKey::Select);
        }
    }

    assert_eq!(hm.len(), 8);
    hm
}
