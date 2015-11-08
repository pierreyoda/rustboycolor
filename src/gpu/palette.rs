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
    pub fn to_rgb(&self) -> RGB {
        PALETTE_CLASSIC_RGB[self.clone() as usize]
    }
}

/// Gives the RGB colors corresponding to the GB's monochrome palette values.
const PALETTE_CLASSIC_RGB: [RGB; 4] = [
    RGB { r: 255, g: 255, b: 255 },
    RGB { r: 192, g: 192, b: 192 },
    RGB { r:  96, g:  96, b:  96 },
    RGB { r:  0 , g:  0 , b:  0  },
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaletteClassic {
    /// The palette's raw byte value.
    raw: u8,
    /// The palette's colors.
    data: [PaletteGrayShade; 4],
}

impl PaletteClassic {
    pub fn new() -> PaletteClassic {
        // TODO : check default palette value
        PaletteClassic{ raw: 0xFF, data: [White, White, White, White] }
    }

    pub fn set(&mut self, value: u8) {
        self.raw     = value;
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

#[cfg(test)]
mod test {
    use super::{PaletteGrayShade, PaletteClassic, PALETTE_CLASSIC_RGB};
    use super::PaletteGrayShade::*;

    #[test]
    fn test_PaletteGrayShade_values() {
        assert_eq!(White as u8, 0b00);
        assert_eq!(LightGray as u8, 0b01);
        assert_eq!(DarkGray as u8, 0b10);
        assert_eq!(Dark as u8, 0b11);
    }

    #[test]
    fn test_PaletteGrayShade_to_RGB() {
        assert_eq!(White.to_rgb(), PALETTE_CLASSIC_RGB[0]);
        assert_eq!(LightGray.to_rgb(), PALETTE_CLASSIC_RGB[1]);
        assert_eq!(DarkGray.to_rgb(), PALETTE_CLASSIC_RGB[2]);
        assert_eq!(Dark.to_rgb(), PALETTE_CLASSIC_RGB[3]);
    }

    #[test]
    fn test_PaletteClassic() {
        let mut palette = PaletteClassic::new();
        palette.set(0b_1011_0001);
        let colors = palette.data();
        assert_eq!(palette.raw(), 0b_1011_0001);
        assert_eq!(colors[0], LightGray);
        assert_eq!(colors[1], White);
        assert_eq!(colors[2], Dark);
        assert_eq!(colors[3], DarkGray);
    }
}
