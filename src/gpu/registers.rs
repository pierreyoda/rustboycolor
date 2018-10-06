use super::GpuMode;

pub const CONTROL: usize = 0xFF40; // LCD Control
pub const STAT: usize = 0xFF41; // LCD Controller status
pub const SCY: usize = 0xFF42;
pub const SCX: usize = 0xFF43;
pub const LY: usize = 0xFF44; // read-only
pub const LYC: usize = 0xFF45;
pub const BGP: usize = 0xFF47; // ignored in CGB mode
pub const OBP_0: usize = 0xFF48; // ignored in CGB mode
pub const OBP_1: usize = 0xFF49; // ignored in CGB mode
pub const WY: usize = 0xFF4A;
pub const WX: usize = 0xFF4B;

pub enum LcdControl {
    BgDisplayEnable = 1 << 0,
    ObjDisplayEnable = 1 << 1,
    ObjSize = 1 << 2,
    BgTileMapDisplaySelect = 1 << 3,
    BgWindowTileDataSelect = 1 << 4,
    WindowDisplayEnable = 1 << 5,
    WindowTileMapDisplaySelect = 1 << 6,
    LcdDisplayEnable = 1 << 7,
}
impl LcdControl {
    pub fn is_set(self, register: u8) -> bool {
        let v = self as u8;
        ((register >> v) & 0x01) == 0x01
    }
}

pub enum LcdControllerStatus {
    HBlankInterrupt = 1 << 3,
    VBlankInterrupt = 1 << 4,
    OamInterrupt = 1 << 5,
    LyCoincidenceInterrupt = 1 << 6,
}
impl LcdControllerStatus {
    pub fn is_set(self, register: u8) -> bool {
        let v = self as u8;
        ((register >> v) & 0x01) == 0x01
    }

    pub fn with_mode(register: u8, mode: GpuMode) -> u8 {
        (register & 0xFC) | (mode as u8)
    }

    /// Set the register's bit 2 to true if LYC=LY, false otherwise.
    pub fn with_coincidence_flag(register: u8, coincidence: bool) -> u8 {
        (register & 0xFB) | if coincidence { 0x01 } else { 0x00 }
    }
}
