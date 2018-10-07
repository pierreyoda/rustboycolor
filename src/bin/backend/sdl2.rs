use std::path::Path;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::mpsc::{Sender, Receiver};
extern crate sdl2;
use self::sdl2::event::Event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::pixels::{Color, PixelFormatEnum};
use self::sdl2::rect::Rect;
use self::sdl2::render::{Texture, TextureCreator, WindowCanvas};
use self::sdl2::video::WindowContext;

use rustboylib::gpu::{RGB, SCREEN_W, SCREEN_H};
use super::{EmulatorBackend, BackendMessage};
use config::EmulatorAppConfig;
use input::get_key_bindings;
use emulator::EmulationMessage;

/// The SDL 2 backend, using rust-sdl2.
pub struct BackendSDL2;

impl BackendSDL2 {
    fn render_display<'a>(tc: &'a TextureCreator<WindowContext>, wc: &mut WindowCanvas,
        frame_buffer: &[RGB], scale_h: u32, scale_v: u32) -> Texture<'a> {
        let (w, w_scale) = (SCREEN_W as i32, scale_h as i32);
        let (h, h_scale) = (SCREEN_H as i32, scale_v as i32);
        let mut texture = tc.create_texture_target(PixelFormatEnum::RGB24,
            (SCREEN_W as u32) * scale_h, (SCREEN_H as u32) * scale_v)
            .expect("BackendSDL2::render_display: texture creation error");
        wc.with_texture_canvas(&mut texture,  |texture_canvas| {
            for y in 0i32..(h as i32) {
                for x in 0i32..(w as i32) {
                    let color = frame_buffer[(y * w  + x) as usize];
                    texture_canvas.set_draw_color(Color::RGB(color.r, color.g, color.b));
                    texture_canvas.fill_rect(Rect::new(x * w_scale, y * h_scale, scale_h, scale_v))
                        .expect("BackendSDL2::render_display: texture canvas fill rect error");
                }
            }
        });
        texture
    }
}

impl EmulatorBackend for BackendSDL2 {
    fn run(&mut self,
           config: EmulatorAppConfig,
           tx: Sender<BackendMessage>,
           rx: Receiver<EmulationMessage>) {
        use emulator::EmulationMessage::*;
        use backend::BackendMessage::*;

        info!("starting the main application thread.");

        // Input bindings
        let key_binds = match get_key_bindings::<Keycode>(&config.get_keyboard_binding(),
                                                          &keycode_from_symbol_hm()) {
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
        let ttf_context = sdl2::ttf::init().unwrap();
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
        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        let texture_creator = canvas.texture_creator();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.present();
        let mut events = sdl_context.event_pump().unwrap();

        let font = ttf_context.load_font(Path::new("assets/OpenSans-Regular.ttf"), 48)
            .expect("could not load 'assets/OpenSans-Regular.ttf'");

        // is the emulation paused ?
        let mut paused = false;
        // avoid spamming 'Event::KeyDown' events for the same key
        let mut last_key: Option<Keycode> = None;

        // FPS count
        let mut fps = 0;
        let mut fps_timer = Instant::now();
        let one_second = Duration::new(1, 0);
        let show_fps = config.get_display_fps();
        let fps_font_color = Color::RGBA(255, 255, 255, 255);
        let fps_target_rect = sdl2::rect::Rect::new(0, 0, 70, 40);
        let fps_surface = font.render("0").blended(fps_font_color).unwrap();
        let mut fps_texture = texture_creator.create_texture_from_surface(&fps_surface).unwrap();

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
                        if last_key.is_some() && keycode == last_key.unwrap() {
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
                                    if let Some(keypad_key) = key_binds.get(&keycode) {
                                        tx.send(KeyDown(*keypad_key)).unwrap();
                                    }
                                }
                            }
                        }
                        last_key = Some(keycode);
                    }
                    Event::KeyUp { keycode: Some(keycode), .. } if !paused => {
                        if let Some(keypad_key) = key_binds.get(&keycode) {
                            tx.send(KeyUp(*keypad_key)).unwrap();
                        }
                        if last_key.is_some() && keycode == last_key.unwrap() {
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
                        UpdateDisplay(frame_buffer) => {
                            let texture = BackendSDL2::render_display(&texture_creator,
                                &mut canvas, &frame_buffer, scale_h as u32, scale_v as u32);
                            canvas.copy(&texture, None, Some(Rect::new(0, 0, w, h)))
                                .expect("BackendSDL2: canvas texture copy error");
                        },
                        Finished => break 'ui,
                    }
                }
                _ => {}
            }

            canvas.clear();

            // FPS count
            fps += 1;
            if fps_timer.elapsed() >= one_second {
                fps_timer = Instant::now();
                let text = format!("{}", fps);
                fps = 0;

                let surface = font.render(&text[..]).blended(fps_font_color).unwrap();
                fps_texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            }
            if show_fps {
                canvas.copy(&fps_texture, None, Some(fps_target_rect)).unwrap();
            }

            canvas.present();
        }

        info!("terminating the main application thread.");
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
    } // F1-F12
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
        let keycode_symbol_hm = super::keycode_from_symbol_hm();
        let key_binds = get_key_bindings::<Keycode>(
            &FromConfigFile("tests/backend_input.toml".into()),
            &keycode_symbol_hm,
        ).unwrap();
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
