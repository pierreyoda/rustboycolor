mod cgb;
mod palette;
mod registers;
mod tile;

use std::cmp;

use crate::cpu::CycleType;
use crate::irq::{Interrupt, IrqHandler};
use crate::memory::Memory;

use self::GpuMode::*;
use self::palette::PaletteClassic;
use self::registers::{LcdControl, LcdControllerInterruptStatus};
use self::tile::Tile;

/// The width of the Game Boy's screen, in pixels.
pub const SCREEN_W: usize = 160;
/// The height of the Game Boy's screen, in pixels.
pub const SCREEN_H: usize = 144;

const BACKGROUND_WIDTH: usize = 256;
const BACKGROUND_HEIGHT: usize = 256;
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
        RGB { r, g, b }
    }
}

/// Type defining the state of a Game Boy (Color) screen, each pixel being
/// fully defined by its RGB color.
pub type ScreenData = [RGB; SCREEN_W * SCREEN_H];

/// The different modes a GPU can spend its time in.
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum GpuMode {
    H_Blank = 0,
    V_Blank = 1,
    OAM_Read = 2,
    VRAM_Read = 3,
}

pub const H_BLANK_CYCLES: CycleType = 204;
pub const V_BLANK_CYCLES: CycleType = 456;
pub const OAM_READ_CYCLES: CycleType = 80;
pub const VRAM_READ_CYCLES: CycleType = 172;

/// The GPU registers' addresses.
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
    /// LCD Control Register
    lcd_control: u8,
    /// LCDC Status Register
    lcdc_status: u8,
    /// The index of the current scanline.
    ///
    /// Can take any value between 0 and 153, with values between 144 and 153
    /// indicating a V-Blank period.
    ///
    /// Writing to the LY register resets it to 0.
    ly: usize,
    /// LY comparison value.
    ///
    /// When both are equal, the coincident bit in the STAT register is set
    /// and (if enabled in the STAT register) an LCD STAT interrupt is requested.
    lyc: usize,
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
    ///
    /// Not used in CGB mode (yet 'Option' is not used for convenience).
    bg_palette: PaletteClassic,
    /// The two object palettes (Classic mode only), assigning gray shades to
    /// the color numbers of the sprites. Color 0 is not used (transparent).
    ///
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
            lcd_control: 0,
            lcdc_status: 0,
            ly: 0,
            lyc: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            frame_buffer: [RGB::new(255, 255, 255); SCREEN_W * SCREEN_H],
            bg_palette: PaletteClassic::new(),
            ob_palettes: [PaletteClassic::new(); 2],
            tileset: [Tile::new([0x00; 16]); 384],
            tilemaps: [[0x00; TILEMAP_SIZE]; 2],
            dirty: true,
        }
    }

    /// Advance the GPU simulation forward by the given amount of clock ticks.
    ///
    /// Loops through OAM_Read, VRAM_Read and H_Blank modes to draw the 144 lines,
    /// then switches to V_Blank mode for 10 lines before starting over.
    pub fn step(&mut self, ticks: CycleType, irq_handler: &mut dyn IrqHandler) {
        use self::LcdControllerInterruptStatus::*;

        if !LcdControl::LcdDisplayEnable.is_set(self.lcd_control) {
            return;
        }

        self.mode_clock += ticks;

        match self.mode {
            // scanline, accessing OAM
            OAM_Read if self.mode_clock >= OAM_READ_CYCLES => {
                self.mode_clock -= OAM_READ_CYCLES;
                self.switch_mode(VRAM_Read);
            }
            // scanline, accessing VRAM
            VRAM_Read if self.mode_clock >= VRAM_READ_CYCLES => {
                self.mode_clock -= VRAM_READ_CYCLES;
                // end of scanline
                self.render_scanline();
                self.switch_mode(H_Blank);
                if HBlank.is_set(self.lcdc_status) {
                    irq_handler.request_interrupt(Interrupt::LCD_Stat);
                }
            }
            // horizontal blank
            H_Blank if self.mode_clock >= H_BLANK_CYCLES => {
                self.mode_clock -= H_BLANK_CYCLES;
                self.ly += 1;
                if self.ly == SCREEN_H {
                    // last H_BLANK: render framebuffer
                    self.switch_mode(V_Blank);
                    self.dirty = true;
                    irq_handler.request_interrupt(Interrupt::V_Blank);
                    if VBlank.is_set(self.lcdc_status) {
                        irq_handler.request_interrupt(Interrupt::LCD_Stat);
                    }
                } else {
                    // move to next line
                    self.switch_mode(OAM_Read);
                    if Oam.is_set(self.lcdc_status) {
                        irq_handler.request_interrupt(Interrupt::LCD_Stat);
                    }
                }
            }
            // vertical blank (10 lines)
            V_Blank if self.mode_clock >= V_BLANK_CYCLES => {
                self.mode_clock -= V_BLANK_CYCLES;
                self.ly += 1;
                if self.ly == SCREEN_H + 10 {
                    // last V_BLANK
                    self.ly = 0;
                    self.switch_mode(OAM_Read);
                    if Oam.is_set(self.lcdc_status) {
                        irq_handler.request_interrupt(Interrupt::LCD_Stat);
                    }
                }
            }
            _ => {}
        }

        // LYC/LY comparison
        if self.lyc == self.ly {
            self.lcdc_status =
                LcdControllerInterruptStatus::with_coincidence_flag(self.lcdc_status, true);
            if LyCoincidence.is_set(self.lcdc_status) {
                irq_handler.request_interrupt(Interrupt::LCD_Stat);
            }
        } else {
            self.lcdc_status =
                LcdControllerInterruptStatus::with_coincidence_flag(self.lcdc_status, false);
        }
    }

    /// Switch the current GPU mode.
    fn switch_mode(&mut self, new_mode: GpuMode) {
        self.lcdc_status =
            LcdControllerInterruptStatus::with_mode(self.lcdc_status, new_mode.clone());
        self.mode = new_mode;
    }

    /// Write the current scanline in the framebuffer.
    fn render_scanline(&mut self) {
        let y = self.ly;
        self.render_line_tiles(y);
        self.render_line_sprites(y);
    }

    fn render_line_tiles(&mut self, y: usize) {
        let palette_data = self.bg_palette.data();
        // background line
        if LcdControl::BgDisplayEnable.is_set(self.lcd_control) {
            for x in 0..SCREEN_W {
                let background_x = (self.scroll_x as usize + x) % BACKGROUND_WIDTH;
                let background_y = (self.scroll_y as usize + y) % BACKGROUND_HEIGHT;

                let tile = self.get_tile(
                    background_x,
                    background_y,
                    LcdControl::BgTileMapDisplaySelect.is_set(self.lcd_control),
                );
                let (x_offset, y_offset) = (background_x % 8, background_y % 8);
                let color_index = tile.data()[y_offset][x_offset] as usize;
                self.frame_buffer[y * SCREEN_W + x] = palette_data[color_index].as_rgb();
            }
        }
        // window line
        if LcdControl::WindowDisplayEnable.is_set(self.lcd_control) {
            let x_start = cmp::max(self.window_x as i32 - 7, 0) as usize;
            for x in x_start..SCREEN_W {
                let window_x = (self.scroll_x as usize + x) % BACKGROUND_WIDTH;
                let window_y = (self.scroll_y as usize + y) % BACKGROUND_HEIGHT;

                let tile = self.get_tile(
                    window_x,
                    window_y,
                    LcdControl::WindowTileMapDisplaySelect.is_set(self.lcd_control),
                );
                let (x_offset, y_offset) = (window_x % 8, window_y % 8);
                let color_index = tile.data()[y_offset][x_offset] as usize;
                self.frame_buffer[y * SCREEN_W + x] = palette_data[color_index].as_rgb();
            }
        }
    }

    fn get_tile(&self, x: usize, y: usize, tilemap_2: bool) -> Tile {
        let index = (y / 8) * 32 + x / 8;
        let tile_index = if tilemap_2 {
            self.tilemaps[1][index]
        } else {
            self.tilemaps[0][index]
        };

        let tileset_index = if LcdControl::BgWindowTileDataSelect.is_set(self.lcd_control) {
            tile_index as usize
        } else {
            256 + (tile_index as usize)
        };
        debug_assert!(
            tileset_index < self.tileset.len(),
            "Gpu.get_tile(x={}, y={}, tilemap_2={}): tileset_index overflow (value={}, max={})",
            x,
            y,
            tilemap_2,
            tileset_index,
            self.tileset.len()
        );
        self.tileset[tileset_index]
    }

    fn render_line_sprites(&mut self, _y: usize) {
        if !LcdControl::ObjDisplayEnable.is_set(self.lcd_control) {
            return;
        }
        // TODO:
    }

    pub fn screen_data(&self) -> Vec<RGB> {
        self.frame_buffer.to_vec()
    }
}

impl Memory for Gpu {
    fn read_byte(&mut self, address: u16) -> u8 {
        use self::cgb::regs as r;
        use self::registers::*;

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
            0x8000..=0x97FF => {
                // tileset
                let addr = a - 0x8000;
                let tile_index = addr / 16;
                let data_index = addr % 16;
                debug_assert!(tile_index < 384);
                self.tileset[tile_index].raw_byte(data_index)
            }
            0x9800..=0x9BFF => {
                // tilemap 0
                let addr = a - 0x9800;
                self.tilemaps[0][addr]
            }
            0x9C00..=0x9FFF => {
                // tilemap 1
                let addr = a - 0x9C00;
                self.tilemaps[1][addr]
            }
            CONTROL => self.lcd_control,
            STAT => self.lcdc_status,
            SCY => self.scroll_y,
            SCX => self.scroll_x,
            LY => self.ly as u8,
            LYC => self.lyc as u8,
            BGP => self.bg_palette.raw(),
            OBP_0 => self.ob_palettes[0].raw(),
            OBP_1 => self.ob_palettes[1].raw(),
            WY => self.window_y,
            WX => self.window_x,
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        use self::cgb::regs as r;
        use self::registers::*;

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
            0x8000..=0x97FF => {
                let addr = a - 0x8000;
                let tile_index = addr / 16;
                let data_index = addr % 16;
                debug_assert!(tile_index < 384);
                self.tileset[tile_index].update_raw_byte(data_index, byte)
            }
            0x9800..=0x9BFF => {
                let addr = a - 0x9800;
                self.tilemaps[0][addr] = byte;
            }
            0x9C00..=0x9FFF => {
                let addr = a - 0x9C00;
                self.tilemaps[1][addr] = byte;
            }
            CONTROL => self.lcd_control = byte,
            // LCDC Status: bits 2 to 0 are read-only
            STAT => self.lcdc_status = (byte & 0xF8) | (self.lcdc_status & 0x07),
            SCY => self.scroll_y = byte,
            SCX => self.scroll_x = byte,
            LY => self.ly = 0,
            LYC => self.lyc = byte as usize,
            BGP => self.bg_palette.set(byte),
            OBP_0 => self.ob_palettes[0].set(byte),
            OBP_1 => self.ob_palettes[1].set(byte),
            WY => self.window_y = byte,
            WX => self.window_x = byte,
            _ => (),
        }
    }
}
