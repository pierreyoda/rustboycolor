mod palette;

use super::cpu::CycleType;
use super::memory::Memory;
use super::irq::{Interrupt, IrqHandler};

use self::GpuMode::*;
use self::palette::{PaletteClassic, PaletteGrayShade};

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

/// The GPU registers' addresses.
mod Regs {
    pub const STAT  : usize = 0xFF41;
    pub const SCY   : usize = 0xFF42;
    pub const SCX   : usize = 0xFF43;
    pub const LY    : usize = 0xFF44; // read-only
    pub const LYC   : usize = 0xFF45;
    pub const BGP   : usize = 0xFF47;
    pub const OBP_0 : usize = 0xFF48;
    pub const OBP_1 : usize = 0xFF49;
    pub const WY    : usize = 0xFF4A;
    pub const WX    : usize = 0xFF4B;
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
    /// Can take any value between 0 and 153, with values between 144 and 153
    /// indicating a V-Blank period.
    /// Writing to the LY register resets it to 0.
    ly: usize,
    /// Horizontal position of the top-left corner of the on-screen background.
    scroll_x: u8,
    /// Vertical position of the top-left corner of the on-screen background.
    scroll_y: u8,
    // Horizontal position of the top-left of the Window area.
    window_x: u8,
    // Vertical position of the top-left of the Window area.
    window_y: u8,
    /// The frame buffer containing the display's pixels.
    frame_buffer: ScreenData,
    /// The background palette, assigning gray shades to the color numbers
    /// of the background and window tiles.
    bg_palette: PaletteClassic,
    /// The two object palettes (Classic mode only), assigning gray shades to
    /// the color numbers of the sprites. Color 0 is not used (transparent).
    ob_palettes: [PaletteClassic; 2],
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
            window_x: 0,
            window_y: 0,
            frame_buffer: [RGB::new(255, 255, 255); SCREEN_W * SCREEN_H],
            bg_palette: PaletteClassic::new(),
            ob_palettes: [PaletteClassic::new(), PaletteClassic::new()],
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
    {
        use self::Regs::*;

        let a = address as usize;
        match a {
            SCY   => self.scroll_y,
            SCX   => self.scroll_x,
            LY    => self.ly as u8,
            BGP   => self.bg_palette.raw(),
            OBP_0 => self.ob_palettes[0].raw(),
            OBP_1 => self.ob_palettes[1].raw(),
            WY    => self.window_y,
            WX    => self.window_x,
            _     => 0,
        }
    }
    fn write_byte(&mut self, address: u16, byte: u8)
    {
        use self::Regs::*;

        let a = address as usize;
        match a {
            SCY   => self.scroll_y = byte,
            SCX   => self.scroll_x = byte,
            LY    => self.ly = 0,
            BGP   => self.bg_palette.set(byte),
            OBP_0 => self.ob_palettes[0].set(byte),
            OBP_1 => self.ob_palettes[1].set(byte),
            WY    => self.window_y = byte,
            WX    => self.window_x = byte,
            _     => (),
        }
    }
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