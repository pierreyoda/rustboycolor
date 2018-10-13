/// Number of sprites stored in the Object Attribute Memory.
pub const OAM_SIZE: usize = 40;

/// A sprite is a movable tile of 8x8 or 8x16 pixels stored in 4 bytes (by order
/// of field definition in the structure).
///
/// Only 10 sprites can be displayed per scanline.
/// In non-CGB mode when x coordinates are different the sprite with the lowest
/// X coordinate has priority and will appear above others.
/// In CGB mode or when the x coordinates are the same the OAM table ordering
/// sets the priority (lowest memory address = highest priority).
#[derive(Copy, Clone)]
pub struct Sprite {
    /// Vertical position on the screen (minus 16).
    /// An offscreen value (y == 0 || y >= 160) hides the sprite.
    y: u8,
    /// Horizontal position on the screen (minus 8).
    /// An offscreen value (x == 0 || x >= 168) hides the sprite, but it will still
    /// affect priority ordering.
    x: u8,
    /// The sprite's tile number, to select a tile in the 0x8000..0x8FFF range.
    /// In 8x16 mode, the upper 8x8 tile is 'tile_number & 0xFE' and the lower
    /// 8x8 tile is 'tile_number | 0x01'.
    tile_number: u8,
    /// The sprite's attributes.
    flags: SpriteFlags,
}

impl Sprite {
    pub fn new() -> Self {
        Sprite {
            x: 0x00,
            y: 0x00,
            tile_number: 0,
            flags: SpriteFlags::empty(),
        }
    }

    /// Read a raw byte of the sprite's data.
    pub fn read_data(&self, index: usize) -> u8 {
        match index {
            0 => self.y,
            1 => self.x,
            2 => self.tile_number,
            3 => self.flags.bits(),
            _ => unreachable!(),
        }
    }

    /// Write a raw byte of the sprite's data.
    pub fn write_data(&mut self, index: usize, data: u8) {
        match index {
            0 => self.y = data,
            1 => self.x = data,
            2 => self.tile_number = data,
            3 => self.flags = SpriteFlags::from_bits_truncate(data),
            _ => unreachable!(),
        }
    }

    pub fn x(&self) -> u8 {
        self.x
    }
    pub fn y(&self) -> u8 {
        self.y
    }
    pub fn tile_number(&self) -> u8 {
        self.tile_number
    }
    pub fn flags(&self) -> SpriteFlags {
        self.flags
    }
}

bitflags! {
    // TODO: CGB flags
    pub struct SpriteFlags: u8 {
        /// Palette selection bit, for classic (non-CGB) mode only.
        /// 0 => OBJ palette #0 / 1 => OBJ palette #1
        const PALETTE  = 0b_0001_0000;
        /// When 1, horizontally flip the sprite.
        const FLIP_X   = 0b_0010_0000;
        /// When 1, vertically flip the sprite.
        const FLIP_Y   = 0b_0100_0000;
        /// Above background when 0, and below background (except White color) when 1.
        const PRIORITY = 0b_1000_0000;
    }
}
