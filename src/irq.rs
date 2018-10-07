/// Interrupt Flag Register memory address.
pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;
/// Interrupt Enable Register memory address.
pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;

/// Handler trait for Interrupt Requests.
pub trait IrqHandler {
    fn request_interrupt(&mut self, interrupt: Interrupt);
}

/// Mock 'IrqHandler'.
pub struct EmptyIrqHandler;

impl IrqHandler for EmptyIrqHandler {
    fn request_interrupt(&mut self, _: Interrupt) {}
}

/// The different interrupts used in the Game Boy.
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Interrupt {
    V_Blank = 1 << 0,
    LCD_Stat = 1 << 1,
    Timer = 1 << 2,
    Serial = 1 << 3,
    Joypad = 1 << 4,
}

impl Interrupt {
    /// Try to build an 'Interrupt' instance from the associated byte value.
    pub fn from_u8(byte: u8) -> Option<Interrupt> {
        match byte {
            0x01 => Some(Interrupt::V_Blank),
            0x02 => Some(Interrupt::LCD_Stat),
            0x04 => Some(Interrupt::Timer),
            0x08 => Some(Interrupt::Serial),
            0x10 => Some(Interrupt::Joypad),
            _ => None,
        }
    }

    /// Get the address the CPU will jump to to handle the interrupt.
    pub fn address(&self) -> u16 {
        match *self {
            Interrupt::V_Blank => 0x40,
            Interrupt::LCD_Stat => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Serial => 0x58,
            Interrupt::Joypad => 0x60,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Interrupt;
    use super::Interrupt::*;

    #[test]
    fn test_irq_address() {
        assert_eq!(V_Blank.address(), 0x40);
        assert_eq!(LCD_Stat.address(), 0x48);
        assert_eq!(Timer.address(), 0x50);
        assert_eq!(Serial.address(), 0x58);
        assert_eq!(Joypad.address(), 0x60);
    }

    #[test]
    fn test_irq_from_u8() {
        assert_eq!(Interrupt::from_u8(1 << 0), Some(V_Blank));
        assert_eq!(Interrupt::from_u8(1 << 1), Some(LCD_Stat));
        assert_eq!(Interrupt::from_u8(1 << 2), Some(Timer));
        assert_eq!(Interrupt::from_u8(1 << 3), Some(Serial));
        assert_eq!(Interrupt::from_u8(1 << 4), Some(Joypad));
    }
}
