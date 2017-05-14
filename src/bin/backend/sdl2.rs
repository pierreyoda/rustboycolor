use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
extern crate sdl2;
use self::sdl2::event::Event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::pixels::Color;

use rustboylib::gpu::{SCREEN_W, SCREEN_H};
use super::{EmulatorBackend, BackendMessage};
use config::EmulatorAppConfig;
use input::get_key_bindings;
use emulator::EmulationMessage;

/// The SDL 2 backend, using rust-sdl2.
pub struct BackendSDL2;

impl EmulatorBackend for BackendSDL2 {
    fn run(&mut self,
           config: EmulatorAppConfig,
           tx: Sender<BackendMessage>,
           rx: Receiver<EmulationMessage>) {
        use emulator::EmulationMessage::*;
        use backend::BackendMessage::*;

        info!("starting the main application thread.");

        // Input bindings
        let key_binds = match get_key_bindings::<Keycode>(config.get_keyboard_binding(),
                                                          keycode_from_symbol_hm()) {
            Ok(hm) => hm,
            Err(why) => {
                error!("SDL2 backend input : {}", why);
                return;
            }
        };

        // Window size
        let (scale_h, scale_v) = config.compute_display_scale();
        let w = (SCREEN_W as u32) * (scale_h as u32);
        let h = (SCREEN_H as u32) * (scale_v as u32);
        info!("display scale = ({}, {}).", scale_h, scale_v);

        // SDL 2 initialization
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = match video_subsystem.window(config.get_title(), w, h)
                                          .position_centered()
                                          .opengl()
                                          .build() {
            Ok(window) => window,
            Err(why) => {
                error!("SDL2 backend failed to create the window : {}", why);
                return;
            }
        };
        let mut renderer = match window.renderer().build() {
            Ok(renderer) => renderer,
            Err(why) => {
                error!("SDL2 backend failed to create the renderer : {}", why);
                return;
            }
        };
        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.present();
        let mut events = sdl_context.event_pump().unwrap();

        // is the emulation paused ?
        let mut paused = false;
        // avoid spamming 'Event::KeyDown' events for the same key
        let mut last_key: Option<Keycode> = None;

        // Main loop
        'ui: loop {
            // Event loop
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        paused = true;
                        tx.send(Quit).unwrap();
                    }
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        if !last_key.is_none() && keycode == last_key.unwrap() {
                            continue;
                        }
                        match keycode {
                            // quit
                            Keycode::Escape => {
                                paused = true;
                                tx.send(Quit).unwrap();
                            }
                            // toggle pause
                            Keycode::Return => {
                                paused = !paused;
                                tx.send(UpdateRunStatus(paused)).unwrap();
                            }
                            _ => {
                                if !paused {
                                    match key_binds.get(&keycode) {
                                        Some(keypad_key) => {
                                            tx.send(KeyDown(*keypad_key)).unwrap();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        last_key = Some(keycode);
                    }
                    Event::KeyUp { keycode: Some(keycode), .. } if !paused => {
                        match key_binds.get(&keycode) {
                            Some(keypad_key) => {
                                tx.send(KeyUp(*keypad_key)).unwrap();
                            }
                            _ => {}
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
                Ok(emulation_message) => {
                    match emulation_message {
                        Finished => break 'ui,
                    }
                }
                _ => {}
            }
        }

        info!("terminating the main application thread.")
    }
}

pub fn keycode_from_symbol_hm() -> HashMap<String, Keycode> {
    let mut hm = HashMap::new();

    hm.insert("Up".into(), Keycode::Up);
    hm.insert("Down".into(), Keycode::Down);
    hm.insert("Left".into(), Keycode::Left);
    hm.insert("Right".into(), Keycode::Right);
    hm.insert("Numpad0".into(), Keycode::Kp0);
    hm.insert("Numpad1".into(), Keycode::Kp1);
    hm.insert("Numpad2".into(), Keycode::Kp2);
    hm.insert("Numpad3".into(), Keycode::Kp3);
    hm.insert("Numpad4".into(), Keycode::Kp4);
    hm.insert("Numpad5".into(), Keycode::Kp5);
    hm.insert("Numpad6".into(), Keycode::Kp6);
    hm.insert("Numpad7".into(), Keycode::Kp7);
    hm.insert("Numpad8".into(), Keycode::Kp8);
    hm.insert("Numpad9".into(), Keycode::Kp9);
    hm.insert("NumpadPlus".into(), Keycode::KpPlus);
    hm.insert("NumpadMinus".into(), Keycode::KpMinus);
    hm.insert("NumpadMultiply".into(), Keycode::KpMultiply);
    hm.insert("NumpadDivide".into(), Keycode::KpDivide);
    // reference : https://wiki.libsdl.org/SDL_Keycode
    // and : "keycode.rs" from https://github.com/AngryLawyer/rust-sdl2/
    let mut sdl2_key_names = Vec::<String>::new();
    for c in b'A'..b'Z' + 1 {
        sdl2_key_names.push((c as char).to_string());
    }
    for i in 1..13 {
        sdl2_key_names.push(format!("F{}", i));
    } // F0-F12
    for key_name in sdl2_key_names {
        let key_code = match Keycode::from_name(&key_name[..]) {
            Some(code) => code,
            None => panic!("SDL2 backend : invalid keycode \"{}\"", key_name),
        };
        hm.insert(key_name.clone(), key_code);
    }

    hm
}

#[cfg(test)]
mod test {
    use super::sdl2::keyboard::Keycode;
    use rustboylib::joypad::JoypadKey;
    use input::get_key_bindings;
    use input::KeyboardBinding::FromConfigFile;

    #[test]
    fn test_keyboard_hm_from_config() {
        let key_binds = get_key_bindings::<Keycode>(FromConfigFile("tests/backend_input.toml"
                                                                       .into()),
                                                    super::keycode_from_symbol_hm())
                            .unwrap();
        assert_eq!(*key_binds.get(&Keycode::Up).unwrap(), JoypadKey::Up);
        assert_eq!(*key_binds.get(&Keycode::Down).unwrap(), JoypadKey::Down);
        assert_eq!(*key_binds.get(&Keycode::Left).unwrap(), JoypadKey::Left);
        assert_eq!(*key_binds.get(&Keycode::Right).unwrap(), JoypadKey::Right);
        assert_eq!(*key_binds.get(&Keycode::Kp1).unwrap(), JoypadKey::Select);
        assert_eq!(*key_binds.get(&Keycode::Kp3).unwrap(), JoypadKey::Start);
        assert_eq!(*key_binds.get(&Keycode::E).unwrap(), JoypadKey::A);
        assert_eq!(*key_binds.get(&Keycode::T).unwrap(), JoypadKey::B);
    }
}
