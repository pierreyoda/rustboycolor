use std::sync::mpsc::{Sender, Receiver};
extern crate sdl2;
use self::sdl2::render::RenderDrawer;
use self::sdl2::event::Event;
use self::sdl2::rect::Rect;
use self::sdl2::pixels::Color;
use self::sdl2::keycode::KeyCode;

use rustboylib::gpu::{SCREEN_W, SCREEN_H};
use super::{EmulatorBackend, BackendMessage};
use config::EmulatorAppConfig;
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
        let mut drawer = renderer.drawer();
        drawer.set_draw_color(Color::RGB(0, 0, 0));
        drawer.present();
        let mut events = context.event_pump();

        // Loop variables
        let mut paused = false;

        // Main loop
        'ui: loop {
            // Event loop
            for event in events.poll_iter() {
                match event {
                    Event::Quit {..} => { paused = true; tx.send(Quit); },
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
