mod cgb;
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

const TILEMAP_SIZE: usize = 0x400;

/// Simple RGB color representation.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r: r, g: g, b: b }
    }
}

/// Type defining the state of a Game Boy (Color) screen, each pixel being
/// fully defined by its RGB color.
pub type ScreenData = [RGB; SCREEN_W * SCREEN_H];

/// A tile is an area of 8x8 pixels.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    data: [[u8; 8]; 8],
}

impl Tile {
    pub fn new(raw_data: [u8; 16]) -> Tile {
        let mut new_tile = Tile {
            raw_data: raw_data,
            data: [[0x00; 8]; 8],
        };
        new_tile.update();
        new_tile
    }

    /// Update the tile's internal state to match its raw value.
    pub fn update(&mut self) {
        for y in 0..8 {
            let line_lo = self.raw_data[y * 2];
            let line_hi = self.raw_data[y * 2 + 1];
            for x in 0..8 {
                let color = ((line_hi >> (7 - x)) & 0x01) << 1 | ((line_lo >> (7 - x)) & 0x01);
                debug_assert!(color < 4);
                self.data[y][x] = color;
            }
        }
    }

    pub fn data(&self) -> &[[u8; 8]; 8] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [[u8; 8]; 8] {
        &mut self.data
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
mod regs {
    pub const STAT: usize = 0xFF41;
    pub const SCY: usize = 0xFF42;
    pub const SCX: usize = 0xFF43;
    pub const LY: usize = 0xFF44; // read-only
    pub const LYC: usize = 0xFF45;
    pub const BGP: usize = 0xFF47; // ignored in CGB mode
    pub const OBP_0: usize = 0xFF48; // ignored in CGB mode
    pub const OBP_1: usize = 0xFF49; // ignored in CGB mode
    pub const WY: usize = 0xFF4A;
    pub const WX: usize = 0xFF4B;
}

/// The structure holding and emulating the GPU state.
///
/// Time durations are expressed in CPU clock cycles with a CPU clock speed of
/// 4194304 Hz.
pub struct Gpu {
    /// If true, a Game Boy Color GPU will be emulated.
    cgb_mode: bool,
    /// CGB-specific data, needed in CGB mode.
    cgb_data: Option<cgb::GpuData>,
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
    /// Not used in CGB mode (yet 'Option' is not used for convenience).
    bg_palette: PaletteClassic,
    /// The two object palettes (Classic mode only), assigning gray shades to
    /// the color numbers of the sprites. Color 0 is not used (transparent).
    /// Not used in CGB mode.
    ob_palettes: [PaletteClassic; 2],
    /// The tileset in VRAM.
    tileset: [Tile; 384],
    /// The two tilemaps in VRAM.
    tilemaps: [[u8; TILEMAP_SIZE]; 2],
    /// Should the screen be redrawn by the frontend ?
    /// Must be externally set to false after that.
    pub dirty: bool,
}

impl Gpu {
    /// Create and return a new 'Gpu' instance.
    pub fn new(cgb_mode: bool) -> Gpu {
        Gpu {
            cgb_mode,
            cgb_data: if cgb_mode {
                Some(cgb::GpuData::new())
            } else {
                None
            },
            mode: H_Blank,
            mode_clock: 0,
            ly: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            frame_buffer: [RGB::new(255, 255, 255); SCREEN_W * SCREEN_H],
            bg_palette: PaletteClassic::new(),
            ob_palettes: [PaletteClassic::new(); 2],
            tileset: [Tile::new([0x00; 16]); 384],
            tilemaps: [[0x00; TILEMAP_SIZE]; 2],
            dirty: false,
        }
    }

    /// Advance the GPU simulation.
    pub fn step(&mut self, ticks: CycleType, irq_handler: &mut IrqHandler) {
        self.mode_clock += ticks;

        match self.mode {
            // scanline, accessing OAM
            OAM_Read => {
                if self.mode_clock >= 80 {
                    self.switch_mode(VRAM_Read)
                }
            }
            // scanline, accessing VRAM
            VRAM_Read => {
                if self.mode_clock >= 172 {
                    // end of scanline
                    self.switch_mode(H_Blank);
                    self.render_scanline();
                    // TODO : throw LCD_Stat interrupt here ?
                }
            }
            // horizontal blank
            H_Blank => {
                if self.mode_clock >= 204 {
                    self.ly += 1;
                    if self.ly == SCREEN_H {
                        self.switch_mode(V_Blank);
                        self.dirty = true; // the framebuffer can be rendered
                        irq_handler.request_interrupt(Interrupt::V_Blank);
                    } else {
                        self.switch_mode(OAM_Read);
                    }
                }
            }
            // vertical blank (10 lines)
            V_Blank => {
                if self.mode_clock >= 456 {
                    self.mode_clock = 0;
                    self.ly += 1;
                    if self.ly == SCREEN_H + 10 {
                        self.switch_mode(OAM_Read);
                        self.ly = 0;
                    }
                }
            }
        }
    }

    /// Switch the current GPU mode.
    fn switch_mode(&mut self, new_mode: GpuMode) {
        self.mode_clock = 0;
        self.mode = new_mode;
    }

    /// Write the current scanline in the framebuffer.
    fn render_scanline(&mut self) {}
}

impl Memory for Gpu {
    fn read_byte(&mut self, address: u16) -> u8 {
        use self::regs::*;
        use self::cgb::regs as r;

        let a = address as usize;

        if self.cgb_mode {
            let data = self.cgb_data.as_ref().unwrap();
            match a {
                r::VRAM_BANK => return data.vram_bank_selector,
                r::BGP_INDEX => return data.bg_palette_index.raw_value(),
                r::BGP_DATA => return data.get_bg_color(),
                r::OBP_INDEX => return data.ob_palette_index.raw_value(),
                r::OBP_DATA => return data.get_ob_color(),
                _ => {}
            }
        }
        match a {
            0x8000...0x9FFF => {
                // Video RAM
                let bank_address = a & 0x1FFF;
                if self.cgb_mode {
                    let data = self.cgb_data.as_ref().unwrap();
                    if data.vram_bank_selector & 0x01 == 0x01 {
                        // return data.vram_bank[bank_address];
                    }
                }
                // return self.vram_bank[bank_address];
                0
            }
            SCY => self.scroll_y,
            SCX => self.scroll_x,
            LY => self.ly as u8,
            BGP => self.bg_palette.raw(),
            OBP_0 => self.ob_palettes[0].raw(),
            OBP_1 => self.ob_palettes[1].raw(),
            WY => self.window_y,
            WX => self.window_x,
            _ => 0,
        }
    }
    fn write_byte(&mut self, address: u16, byte: u8) {
        use self::regs::*;
        use self::cgb::regs as r;

        let a = address as usize;

        if self.cgb_mode {
            let data = self.cgb_data.as_mut().unwrap();
            let mut done = true;
            match a {
                r::VRAM_BANK => data.vram_bank_selector = byte,
                r::BGP_INDEX => data.bg_palette_index.update_with(byte),
                r::BGP_DATA => data.set_bg_color(byte),
                r::OBP_INDEX => data.ob_palette_index.update_with(byte),
                r::OBP_DATA => data.set_ob_color(byte),
                _ => {
                    done = false;
                }
            }
            if done {
                return;
            }
        }
        match a {
            0x8000...0x9FFF => {
                // Video RAM
                let bank_address = a & 0x1FFF;
                if self.cgb_mode {
                    let data = self.cgb_data.as_mut().unwrap();
                    if data.vram_bank_selector & 0x01 == 0x01 {
                        // data.vram_bank[bank_address] = byte;
                    }
                }
                // self.vram_bank[bank_address] = byte;
            }
            SCY => self.scroll_y = byte,
            SCX => self.scroll_x = byte,
            LY => self.ly = 0,
            BGP => self.bg_palette.set(byte),
            OBP_0 => self.ob_palettes[0].set(byte),
            OBP_1 => self.ob_palettes[1].set(byte),
            WY => self.window_y = byte,
            WX => self.window_x = byte,
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Tile;

    #[test]
    fn test_tile_update() {
        // Source for the Tile example: http://fms.komkon.org/GameBoy/Tech/Software.html
        // .33333..                          .33333.. -> 0b0111_1100 -> 0x7C
        // 22...22.                                      0b0111_1100 -> 0x7C
        // 11...11.                          22...22. -> 0b0000_0000 -> 0x00
        // 2222222.                                      0b1100_0110 -> 0xC6
        // 33...33.                          11...11. -> 0b1100_0110 -> 0xC6
        // 22...22.                                      0b0000_0000 -> 0x00
        // 11...11.                          2222222. -> 0b0000_0000 -> 0x00
        // ........                                      0b1111_1110 -> 0xFE
        // 33...33. -> 0b1100_0110 -> 0xC6
        // 0b1100_0110 -> 0xC6
        // 22...22. -> 0b0000_0000 -> 0x00
        // 0b1100_0110 -> 0xC6
        // 11...11. -> 0b1100_0110 -> 0xC6
        // 0b0000_0000 -> 0x00
        // ........ -> 0b0000_0000 -> 0x00
        // 0b0000_0000 -> 0x00
        //
        let tile = Tile::new([0x7C, 0x7C, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0xFE, 0xC6, 0xC6, 0x00,
                              0xC6, 0xC6, 0x00, 0x00, 0x00]);
        let data = tile.data();
        let data_test = [[0, 3, 3, 3, 3, 3, 0, 0],
                         [2, 2, 0, 0, 0, 2, 2, 0],
                         [1, 1, 0, 0, 0, 1, 1, 0],
                         [2, 2, 2, 2, 2, 2, 2, 0],
                         [3, 3, 0, 0, 0, 3, 3, 0],
                         [2, 2, 0, 0, 0, 2, 2, 0],
                         [1, 1, 0, 0, 0, 1, 1, 0],
                         [0, 0, 0, 0, 0, 0, 0, 0]];
        for (y, line) in data.iter().enumerate() {
            for (x, color_value) in line.iter().enumerate() {
                assert_eq!(*color_value, data_test[y][x]);
            }
        }
    }
}
