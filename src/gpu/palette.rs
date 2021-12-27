use super::RGB;

use self::PaletteGrayShade::*;

/// The 4 shades of grey that the Game Boy (Classic)'s monochrome LCD can
/// display.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PaletteGrayShade {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Dark = 3,
}

impl PaletteGrayShade {
    /// Build a 'PaletteGrayShade' value from a byte.
    /// Assumption : value <= 3 (see 'PaletteClassic' usage for justification).
    pub fn from_u8(value: u8) -> PaletteGrayShade {
        match value {
            0 => White,
            1 => LightGray,
            2 => DarkGray,
            3 => Dark,
            _ => unreachable!(),
        }
    }

    /// Get the RGB color corresponding to the palette value.
    pub fn as_rgb(&self) -> RGB {
        PALETTE_CLASSIC_RGB[*self as usize]
    }
}

/// Gives the RGB colors corresponding to the GB's monochrome palette values.
const PALETTE_CLASSIC_RGB: [RGB; 4] = [
    RGB {
        r: 255,
        g: 255,
        b: 255,
    },
    RGB {
        r: 192,
        g: 192,
        b: 192,
    },
    RGB {
        r: 96,
        g: 96,
        b: 96,
    },
    RGB { r: 0, g: 0, b: 0 },
];

/// The palette in the Game Boy (Classic) allows by changing a single byte to
/// individually assign 4 colors to arbitrary 'PaletteGrayShade' values :
///
/// bits 7-6 : shade for color 3
/// bits 5-4 : shade for color 2
/// bits 3-2 : shade for color 1
/// bits 1-0 : shade for color 0
///
/// A shade is thus coded as two bits to directly map to the 'PaletteGrayShade'
/// values : 0 for white, 1 for light gray, 2 for dark gray and 3 for dark.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PaletteClassic {
    /// The palette's raw byte value.
    raw: u8,
    /// The palette's colors.
    data: [PaletteGrayShade; 4],
}

impl PaletteClassic {
    pub fn new() -> PaletteClassic {
        // TODO: check default palette value
        PaletteClassic {
            raw: 0xFF,
            data: [White, White, White, White],
        }
    }

    pub fn set(&mut self, value: u8) {
        self.raw = value;
        self.data[0] = PaletteGrayShade::from_u8((value >> 0) & 0b11);
        self.data[1] = PaletteGrayShade::from_u8((value >> 2) & 0b11);
        self.data[2] = PaletteGrayShade::from_u8((value >> 4) & 0b11);
        self.data[3] = PaletteGrayShade::from_u8((value >> 6) & 0b11);
    }

    pub fn raw(&self) -> u8 {
        self.raw
    }

    pub fn data(&self) -> &[PaletteGrayShade; 4] {
        &self.data
    }
}

/// A color in the GameBoyColor is defined by 15 bits (low byte : 0-7 bits,
/// high byte : 8-15 bits) as such:
/// bit 0-4: red intensity (so the possible values are : 00-1F)
/// bit 5-9: green intensity
/// bit 10-14: blue intensity
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PaletteColorValue {
    /// The color's raw value.
    raw: u16,
    /// The color's RGB value, stored for more efficient drawing.
    rgb: RGB,
}

impl PaletteColorValue {
    pub fn new(raw_value: u16) -> PaletteColorValue {
        PaletteColorValue {
            raw: raw_value,
            rgb: PaletteColorValue::compute_rgb(raw_value),
        }
    }

    pub fn set(&mut self, raw_value: u16) {
        self.raw = raw_value;
        self.rgb = PaletteColorValue::compute_rgb(raw_value);
    }

    pub fn set_low(&mut self, byte: u8) {
        let new_raw = (self.raw & 0xFF00) | (byte as u16);
        self.set(new_raw);
    }
    pub fn set_high(&mut self, byte: u8) {
        let new_raw = (self.raw & 0x00FF) | ((byte as u16) << 8);
        self.set(new_raw);
    }

    pub fn raw_low(&self) -> u8 {
        (self.raw & 0x00FF) as u8
    }
    pub fn raw_high(&self) -> u8 {
        (self.raw >> 8) as u8
    }

    pub fn rgb(&self) -> RGB {
        self.rgb
    }

    fn compute_rgb(raw_value: u16) -> RGB {
        // the color values are on 5 bits, which means 32 values
        // we thus need to multiply by 8 to reach the 256 RGB scale
        RGB {
            r: (((raw_value >> 0) & 0x001F) as u8) * 8,
            g: (((raw_value >> 5) & 0x001F) as u8) * 8,
            b: (((raw_value >> 10) & 0x001F) as u8) * 8,
        }
    }
}

/// A GameBoyColor palette is defined by its 4 'PaletteColorValue', meaning
/// 4 * 2 = 8 bytes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PaletteColor {
    data: [PaletteColorValue; 4],
}

impl PaletteColor {
    pub fn new() -> PaletteColor {
        // TODO: check default palette value
        PaletteColor {
            data: [PaletteColorValue::new(0x0000); 4],
        }
    }

    pub fn data(&self) -> &[PaletteColorValue; 4] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [PaletteColorValue; 4] {
        &mut self.data
    }
}

#[cfg(test)]
mod test {
    use super::PaletteGrayShade::*;
    use super::{PaletteClassic, PaletteColorValue, PALETTE_CLASSIC_RGB};

    #[test]
    fn test_palette_gray_shade_values() {
        assert_eq!(White as u8, 0b00);
        assert_eq!(LightGray as u8, 0b01);
        assert_eq!(DarkGray as u8, 0b10);
        assert_eq!(Dark as u8, 0b11);
    }

    #[test]
    fn test_palette_gray_shade_to_rgb() {
        assert_eq!(White.as_rgb(), PALETTE_CLASSIC_RGB[0]);
        assert_eq!(LightGray.as_rgb(), PALETTE_CLASSIC_RGB[1]);
        assert_eq!(DarkGray.as_rgb(), PALETTE_CLASSIC_RGB[2]);
        assert_eq!(Dark.as_rgb(), PALETTE_CLASSIC_RGB[3]);
    }

    #[test]
    fn test_palette_classic() {
        let mut palette = PaletteClassic::new();
        palette.set(0b_1011_0001);
        let colors = palette.data();
        assert_eq!(palette.raw(), 0b_1011_0001);
        assert_eq!(colors[0], LightGray);
        assert_eq!(colors[1], White);
        assert_eq!(colors[2], Dark);
        assert_eq!(colors[3], DarkGray);
    }

    #[test]
    fn test_palette_color_value() {
        let mut color = PaletteColorValue::new(0x0000);
        assert_eq!(color.raw_low(), 0x00);
        assert_eq!(color.raw_high(), 0x00);
        assert_eq!(color.rgb().r, 0);
        assert_eq!(color.rgb().g, 0);
        assert_eq!(color.rgb().b, 0);

        color.set(0xF7A9);
        assert_eq!(color.raw_low(), 0xA9);
        assert_eq!(color.raw_high(), 0xF7);
        assert_eq!(color.rgb().r, 72);
        assert_eq!(color.rgb().g, 232);
        assert_eq!(color.rgb().b, 232);

        color.set_high(0x38);
        color.set_low(0xB2);
        assert_eq!(color.raw_high(), 0x38);
        assert_eq!(color.raw_low(), 0xB2);
        assert_eq!(color.rgb().r, 144);
        assert_eq!(color.rgb().g, 40);
        assert_eq!(color.rgb().b, 112);
    }
}
