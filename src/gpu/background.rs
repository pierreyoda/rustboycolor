/// A tile is an area of 8x8 pixels.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Tile {
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
        //                                   33...33. -> 0b1100_0110 -> 0xC6
        //                                               0b1100_0110 -> 0xC6
        //                                   22...22. -> 0b0000_0000 -> 0x00
        //                                               0b1100_0110 -> 0xC6
        //                                   11...11. -> 0b1100_0110 -> 0xC6
        //                                               0b0000_0000 -> 0x00
        //                                   ........ -> 0b0000_0000 -> 0x00
        //                                               0b0000_0000 -> 0x00
        //
        let tile = Tile::new([
            0x7C, 0x7C, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0xFE,
            0xC6, 0xC6, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0x00,
        ]);
        let data = tile.data();
        let data_test = [
            [0, 3, 3, 3, 3, 3, 0, 0],
            [2, 2, 0, 0, 0, 2, 2, 0],
            [1, 1, 0, 0, 0, 1, 1, 0],
            [2, 2, 2, 2, 2, 2, 2, 0],
            [3, 3, 0, 0, 0, 3, 3, 0],
            [2, 2, 0, 0, 0, 2, 2, 0],
            [1, 1, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];
        for (y, line) in data.iter().enumerate() {
            for (x, color_value) in line.iter().enumerate() {
                assert_eq!(*color_value, data_test[y][x]);
            }
        }
    }
}
