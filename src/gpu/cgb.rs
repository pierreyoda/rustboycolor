use super::palette::PaletteColor;

/// The GameBoyColor-specific GPU register addresses.
pub mod regs {
    pub const VRAM_BANK : usize = 0xFF4F;
    /// Register defining the index of the palette to read/write from 'BGP_DATA'.
    pub const BGP_INDEX : usize = 0xFF68;
    /// Adress from where the background palette of index defined at 'BGP_INDEX'
    /// can be read/modified.
    pub const BGP_DATA  : usize = 0xFF69;
    pub const OBP_INDEX : usize = 0xFF6A;
    pub const OBP_DATA  : usize = 0xFF6B;
}

/// Defines a palette register index (or specification) as following:
/// bit 0   : if 1, read/modify the high byte of the palette color
/// bit 1-2 : number of the color to modify in the palette (0-3)
/// bit 3-5 : index (00-3F)
/// bit 7   : auto increment enabled if 1, else disabled
pub struct PaletteIndexRegister {
    /// The raw value of the index register.
    raw_value: u8,
    /// The index of the palette to read/modify (0-7).
    palette_index: usize,
    /// The number of the color to modify (0-3).
    palette_color_number: usize,
    /// If true, read/write the high byte of a 'PaletteColorValue',
    /// otherwise read/write the low byte.
    high_byte: bool,
    /// If true, automatically increment the palette index after each write
    /// to the 'BGP_DATA' address. Reading has no effect
    auto_increment: bool,
}

impl PaletteIndexRegister {
    pub fn new(raw_value: u8) -> PaletteIndexRegister {
        let mut new_index = PaletteIndexRegister {
            raw_value: 0x00,
            palette_index: 0,
            palette_color_number: 0,
            high_byte: false,
            auto_increment: false,
        };
        new_index.update_with(raw_value);
        new_index
    }

    pub fn raw_value(&self) -> u8 { return self.raw_value }
    pub fn high_byte(&self) -> bool { return self.high_byte }
    pub fn index(&self) -> usize { return self.palette_index }
    pub fn color_index(&self) -> usize { return self.palette_color_number }

    /// Set the register new raw value and update its state to match it.
    pub fn update_with(&mut self, value: u8) {
        self.raw_value = value;
        self.high_byte = (value & 0x01) == 0x01;
        self.palette_color_number = ((value & 0x06) >> 1) as usize;
        self.palette_index = ((value & 0x38) >> 3) as usize;
        self.auto_increment = (value & 0x80) == 0x80;
    }

    /// Must be called every time the associated palette data register is written
    /// to. If auto-increment is set to true (bit 7 = 1), increment the index.
    pub fn auto_increment(&mut self) {
        if self.auto_increment {
            let new_value = self.raw_value+1;
            self.update_with(new_value);
        }
    }
}

/// GameBoyColor-specific GPU data. This allows to eventually save on memory
/// when in classic mode (using an 'Option' typically).
pub struct GpuData {
    /// The background palette index register.
    pub bg_palette_index: PaletteIndexRegister,
    /// The 8 background palettes.
    pub bg_palettes: [PaletteColor; 8],
    /// The object palette index register.
    pub ob_palette_index: PaletteIndexRegister,
    /// The 8 object palettes.
    pub ob_palettes: [PaletteColor; 8],
}

impl GpuData {
    pub fn new() -> GpuData {
        GpuData {
            bg_palette_index: PaletteIndexRegister::new(0x00),
            bg_palettes: [PaletteColor::new(); 8],
            ob_palette_index: PaletteIndexRegister::new(0x00),
            ob_palettes: [PaletteColor::new(); 8],
        }
    }

    /// Get the byte value in the background palette according to the
    /// specifications of the background palette index.
    pub fn get_bg_color(&self) -> u8 {
        let palette = self.bg_palettes[self.bg_palette_index.index()].data();
        if self.bg_palette_index.high_byte() {
            palette[self.bg_palette_index.color_index()].raw_high()
        } else {
            palette[self.bg_palette_index.color_index()].raw_low()
        }
    }
    /// Set the byte value in the background palette according to the
    /// specifications of the background palette index.
    pub fn set_bg_color(&mut self, byte: u8) {
        let palette = self.bg_palettes[self.bg_palette_index.index()].data_mut();
        if self.bg_palette_index.high_byte() {
            palette[self.bg_palette_index.color_index()].set_high(byte);
        } else {
            palette[self.bg_palette_index.color_index()].set_low(byte);
        }
        self.bg_palette_index.auto_increment();
    }

    /// Get the byte value in the object palette according to the
    /// specifications of the object palette index.
    pub fn get_ob_color(&self) -> u8 {
        let palette = self.bg_palettes[self.bg_palette_index.index()].data();
        if self.bg_palette_index.high_byte() {
            palette[self.bg_palette_index.color_index()].raw_high()
        } else {
            palette[self.bg_palette_index.color_index()].raw_low()
        }
    }
    /// Set the byte value in the object palette according to the
    /// specifications of the object palette index.
    pub fn set_ob_color(&mut self, byte: u8) {
        let palette = self.ob_palettes[self.ob_palette_index.index()].data_mut();
        if self.ob_palette_index.high_byte() {
            palette[self.ob_palette_index.color_index()].set_high(byte);
        } else {
            palette[self.ob_palette_index.color_index()].set_low(byte);
        }
        self.ob_palette_index.auto_increment();
    }
}

#[cfg(test)]
mod test {
    use super::PaletteIndexRegister;

    #[test]
    fn test_PaletteIndexRegister_decoding() {
        let index = PaletteIndexRegister::new(0xAD);
        assert_eq!(index.raw_value(), 0xAD);
        assert_eq!(index.high_byte(), true);
        assert_eq!(index.color_index(), 2);
        assert_eq!(index.index(), 5);
    }
}
