mod palette;

use super::cpu::CycleType;
use super::memory::Memory;
use super::irq::{Interrupt, IrqHandler};

use self::GpuMode::*;
use self::palette::PaletteGrayShade;

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

/// A tile is an area of 8x8 pixels.
struct Tile {
    /// Each tile occupies 16 bytes, where 2 bytes represent a line:
    /// byte 0-1 = first line (upper 8 pixels)
    /// byte 2-3 = second lines
    /// etc.
    /// For each line, the first byte defines the bit 0 of the color numbers
    /// and the second byte defines the bit 1. In both cases, bit 7 is the
    /// leftmost pixel and bit 0 the rightmost.
    /// Each pixel thus has a color number from 0 to 3 wich is translated into
    /// colors or shades of gray according to the current palettes.
    raw_data: [u8; 16],
    /// Cached internal state better suited for rendering.
    /// TODO: color number should work for both Classic and Color variants...
    data: [[PaletteGrayShade; 8]; 8],
}

impl Tile {
    pub fn new(raw_data: [u8; 16]) -> Tile {
        let mut new_tile = Tile {
            raw_data: raw_data,
            data: [[PaletteGrayShade::White; 8]; 8],
        };
        new_tile.update();
        new_tile
    }

    /// Update the tile's internal state to match its raw value.
    pub fn update(&mut self) {
        for y in 0..8 {
            let line_lo = self.raw_data[y*2];
            let line_hi = self.raw_data[y*2+1];
            for x in 0..8 {
                let color = ((line_hi >> (7 - x)) & 0x01)<<1
                    | ((line_lo >> (7 - x)) & 0x01);
                self.data[y][x] = PaletteGrayShade::from_u8(color);
            }
        }
    }

    pub fn data(&self) -> &[[PaletteGrayShade; 8]; 8] {
        &self.data
    }
}

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
    /// Horizontal position of the top-left corner of the on-screen background.
    scroll_x: usize,
    /// Vertical position of the top-left corner of the on-screen background.
    scroll_y: usize,
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
            scroll_x: 0,
            scroll_y: 0,
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
                    self.ly = 0;
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

#[cfg(test)]
mod test {
    use super::Tile;

    #[test]
    fn test_tile_update() {
        /*
        Source for the Tile example: http://fms.komkon.org/GameBoy/Tech/Software.html
        .33333..                          .33333.. -> 0b0111_1100 -> 0x7C
        22...22.                                      0b0111_1100 -> 0x7C
        11...11.                          22...22. -> 0b0000_0000 -> 0x00
        2222222.                                      0b1100_0110 -> 0xC6
        33...33.                          11...11. -> 0b1100_0110 -> 0xC6
        22...22.                                      0b0000_0000 -> 0x00
        11...11.                          2222222. -> 0b0000_0000 -> 0x00
        ........                                      0b1111_1110 -> 0xFE
                                          33...33. -> 0b1100_0110 -> 0xC6
                                                      0b1100_0110 -> 0xC6
                                          22...22. -> 0b0000_0000 -> 0x00
                                                      0b1100_0110 -> 0xC6
                                          11...11. -> 0b1100_0110 -> 0xC6
                                                      0b0000_0000 -> 0x00
                                          ........ -> 0b0000_0000 -> 0x00
                                                      0b0000_0000 -> 0x00
        */
        let tile = Tile::new([0x7C, 0x7C, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0xFE,
            0xC6, 0xC6, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0x00]);
        let data = tile.data();
        let data_test = [
            [0,3,3,3,3,3,0,0],
            [2,2,0,0,0,2,2,0],
            [1,1,0,0,0,1,1,0],
            [2,2,2,2,2,2,2,0],
            [3,3,0,0,0,3,3,0],
            [2,2,0,0,0,2,2,0],
            [1,1,0,0,0,1,1,0],
            [0,0,0,0,0,0,0,0],
        ];
        for (y, line) in data.iter().enumerate() {
            for (x, color_value) in line.iter().enumerate() {
                assert_eq!(*color_value as u8, data_test[y][x]);
            }
        }
    }
}
