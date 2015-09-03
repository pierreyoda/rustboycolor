use super::cpu::CycleType;
use super::memory::Memory;
use super::irq::{Interrupt, IrqHandler};

use self::GpuMode::*;

/// The width of the Game Boy's screen, in pixels.
pub const SCREEN_W: usize = 160;
/// The height of the Game Boy's screen, in pixels.
pub const SCREEN_H: usize = 144;

/// Simple RGB color representation.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB { RGB { r: r, g: g, b: b} }
}

/// Type defining the state of a Game Boy (Color) screen, each pixel being
/// fully defined by its RGB color.
pub type ScreenData = [RGB; SCREEN_W * SCREEN_H];

/// The different modes a GPU can spend its time in.
#[allow(non_camel_case_types)]
enum GpuMode {
    H_Blank = 0,
    V_Blank = 1,
    OAM_Read = 2,
    VRAM_Read = 3,
}

/// The structure holding and emulating the GPU state.
///
/// Time durations are expressed in CPU clock cycles with a CPU clock speed of
/// 4194304 Hz.
pub struct Gpu {
    /// The current mode.
    mode: GpuMode,
    /// The number of cycles spent in the current mode.
    mode_clock: CycleType,
    /// The index of the current scanline.
    ly: usize,
    /// The frame buffer containing the display's pixels.
    frame_buffer: ScreenData,
    /// Should the screen be redrawn by the frontend ?
    /// Must be externally set to false after that.
    pub dirty: bool,
}

impl Gpu {
    /// Create and return a new 'Gpu' instance.
    pub fn new() -> Gpu {
        Gpu {
            mode: H_Blank,
            mode_clock: 0,
            ly: 0,
            frame_buffer: [RGB::new(255, 255, 255); SCREEN_W * SCREEN_H],
            dirty: false,
        }
    }

    /// Advance the GPU simulation.
    pub fn step(&mut self, ticks: CycleType, irq_handler: &mut IrqHandler) {
        self.mode_clock += ticks;

        match self.mode {
            // scanline, accessing OAM
            OAM_Read => if self.mode_clock >= 80 { self.switch_mode(VRAM_Read) },
            // scanline, accessing VRAM
            VRAM_Read => if self.mode_clock >= 172 { // end of scanline
                self.switch_mode(H_Blank);
                self.render_scanline();
                // TODO : throw LCD_Stat interrupt here ?
            },
            // horizontal blank
            H_Blank => if self.mode_clock >= 204 {
                self.ly += 1;
                if self.ly == SCREEN_H {
                    self.switch_mode(V_Blank);
                    self.dirty = true; // the framebuffer can be rendered
                    irq_handler.request_interrupt(Interrupt::V_Blank);
                } else {
                    self.switch_mode(OAM_Read);
                }
            },
            // vertical blank (10 lines)
            V_Blank => if self.mode_clock >= 456 {
                self.mode_clock = 0;
                self.ly += 1;
                if self.ly == SCREEN_H+10 {
                    self.switch_mode(OAM_Read);
                }
            },
        }
    }

    /// Switch the current GPU mode.
    fn switch_mode(&mut self, new_mode: GpuMode) {
        self.mode_clock = 0;
        self.mode = new_mode;
    }

    /// Write the current scanline in the framebuffer.
    fn render_scanline(&mut self) {

    }
}

impl Memory for Gpu {
    fn read_byte(&mut self, address: u16) -> u8
    { 0 }
    fn write_byte(&mut self, address: u16, byte: u8)
    { }
}
