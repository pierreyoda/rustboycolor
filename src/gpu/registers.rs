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

#[derive(Clone)]
pub enum LcdControl {
    BgDisplayEnable = 0,
    ObjDisplayEnable = 1,
    ObjSize = 2,
    BgTileMapDisplaySelect = 3,
    BgWindowTileDataSelect = 4,
    WindowDisplayEnable = 5,
    WindowTileMapDisplaySelect = 6,
    LcdDisplayEnable = 7,
}

impl LcdControl {
    pub fn is_set(&self, register: u8) -> bool {
        let v = self.clone() as usize;
        ((register >> v) & 0x01) == 0x01
    }
}

#[derive(Clone)]
pub enum LcdControllerInterruptStatus {
    HBlank = 3,
    VBlank = 4,
    Oam = 5,
    LyCoincidence = 6,
}

impl LcdControllerInterruptStatus {
    pub fn is_set(&self, register: u8) -> bool {
        let v = self.clone() as usize;
        ((register >> v) & 0x01) == 0x01
    }

    pub fn with_mode(register: u8, mode: GpuMode) -> u8 {
        (register & 0xFC) | (mode as u8)
    }

    /// Set the register's bit 2 to true if LYC=LY, false otherwise.
    pub fn with_coincidence_flag(register: u8, coincidence: bool) -> u8 {
        (register & 0xFB) | if coincidence { 0x04 } else { 0x00 }
    }
}

#[cfg(test)]
mod test {
    use super::LcdControl::*;
    use super::LcdControllerInterruptStatus;
    use super::LcdControllerInterruptStatus::*;

    #[test]
    fn test_lcd_control_is_set() {
        assert!(BgDisplayEnable.is_set(1 << 0));
        assert!(ObjDisplayEnable.is_set(1 << 1));
        assert!(ObjSize.is_set(1 << 2));
        assert!(BgTileMapDisplaySelect.is_set(1 << 3));
        assert!(BgWindowTileDataSelect.is_set(1 << 4));
        assert!(WindowDisplayEnable.is_set(1 << 5));
        assert!(WindowTileMapDisplaySelect.is_set(1 << 6));
        assert!(LcdDisplayEnable.is_set(1 << 7));
    }

    #[test]
    fn test_lcdc_status_is_set() {
        assert!(HBlank.is_set(1 << 3));
        assert!(VBlank.is_set(1 << 4));
        assert!(Oam.is_set(1 << 5));
        assert!(LyCoincidence.is_set(1 << 6));
    }

    #[test]
    fn test_lcdc_status_with_mode() {
        use crate::gpu::GpuMode::*;
        let lcdc_status = 0b_0110_1011;
        assert_eq!(
            LcdControllerInterruptStatus::with_mode(lcdc_status, H_Blank),
            0b_0110_1000
        );
        assert_eq!(
            LcdControllerInterruptStatus::with_mode(lcdc_status, V_Blank),
            0b_0110_1001
        );
        assert_eq!(
            LcdControllerInterruptStatus::with_mode(lcdc_status, OAM_Read),
            0b_0110_1010
        );
        assert_eq!(
            LcdControllerInterruptStatus::with_mode(lcdc_status, VRAM_Read),
            0b_0110_1011
        );
    }

    #[test]
    fn test_lcdc_status_with_coincidence_flag() {
        assert_eq!(
            LcdControllerInterruptStatus::with_coincidence_flag(0b_1011_0011, true),
            0b_1011_0111
        );
        assert_eq!(
            LcdControllerInterruptStatus::with_coincidence_flag(0b_1011_0111, false),
            0b_1011_0011
        );
    }
}
