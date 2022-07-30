use self::JoypadKey::*;
use crate::irq::{Interrupt, IrqHandler};
use crate::memory::Memory;

pub const JOYPAD_ADDRESS: u16 = 0xFF00;
pub const JOYPAD_KEYS: [&str; 8] = ["Up", "Down", "Left", "Right", "Select", "Start", "A", "B"];
pub const JOYPAD_SELECT_DIRECTIONAL: u8 = 1 << 4;
pub const JOYPAD_SELECT_BUTTON: u8 = 1 << 5;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JoypadKey {
    Up,
    Down,
    Right,
    Left,
    A,
    B,
    Select,
    Start,
}

impl JoypadKey {
    /// Build and return a JoypadKey corresponding to the given symbol, if possible.
    pub fn from_string_slice(symbol: &str) -> Option<JoypadKey> {
        match symbol {
            "Up" => Some(JoypadKey::Up),
            "Down" => Some(JoypadKey::Down),
            "Right" => Some(JoypadKey::Right),
            "Left" => Some(JoypadKey::Left),
            "A" => Some(JoypadKey::A),
            "B" => Some(JoypadKey::B),
            "Select" => Some(JoypadKey::Select),
            "Start" => Some(JoypadKey::Start),
            _ => None,
        }
    }
}

/// The structure representing the joypad on the Game Boy (Color).
///
/// A single byte, mapped at the 0xFF00 address in memory, allows to read
/// the user input as following (from highest to lowest bit) :
///
/// 7 : not used
/// 6 : not used
/// 5 : if 0, will read button keys from lowest nibble of row 2
/// 4 : if 0, will read direction keys from lowest nibble of row 1
/// | read-only >>
/// 3 : if 0, down / start button is pressed
/// 2 : if 0, up / select button is pressed
/// 1 : if 0, left / B button is pressed
/// 0 : if 0, right / A button is pressed
///
/// TODO:
/// On a real device, the down and left directions cannot be simultaneously
/// pressed with respectively the up and right directions.
pub struct Joypad {
    /// The 2x4 matrix holding the key states (0 = pressed).
    /// row 1 : direction / row 2 : buttons
    rows: [u8; 2],
    /// The currently selected row (0 = none).
    selection: usize,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            rows: [0x0F, 0x0F],
            selection: 0,
        }
    }
}

impl Joypad {
    pub fn key_down(&mut self, key: &JoypadKey, irq_handler: &mut dyn IrqHandler) {
        match *key {
            Down => self.rows[0] &= 0x07,
            Up => self.rows[0] &= 0x0B,
            Left => self.rows[0] &= 0x0D,
            Right => self.rows[0] &= 0x0E,
            Start => self.rows[1] &= 0x07,
            Select => self.rows[1] &= 0x0B,
            B => self.rows[1] &= 0x0D,
            A => self.rows[1] &= 0x0E,
        }
        irq_handler.request_interrupt(Interrupt::Joypad);
    }

    pub fn key_up(&mut self, key: &JoypadKey) {
        match *key {
            Down => self.rows[0] |= 0x08,
            Up => self.rows[0] |= 0x04,
            Left => self.rows[0] |= 0x02,
            Right => self.rows[0] |= 0x01,
            Start => self.rows[1] |= 0x08,
            Select => self.rows[1] |= 0x04,
            B => self.rows[1] |= 0x02,
            A => self.rows[1] |= 0x01,
        }
    }
}

impl Memory for Joypad {
    fn read_byte(&mut self, address: u16) -> u8 {
        debug_assert!(address == JOYPAD_ADDRESS);
        match self.selection {
            0 => 0x00,
            1 | 2 => self.rows[self.selection - 1],
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        debug_assert!(address == JOYPAD_ADDRESS);
        // filter bits 4 and 5
        self.selection = match byte & 0x30 {
            JOYPAD_SELECT_DIRECTIONAL => 1, // bit 4 = row 1
            JOYPAD_SELECT_BUTTON => 2,      // bit 5 = row 2
            0x30 => 0,
            0x00 => 0,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::JoypadKey::*;
    use super::{
        Joypad, JoypadKey, JOYPAD_ADDRESS, JOYPAD_KEYS, JOYPAD_SELECT_BUTTON,
        JOYPAD_SELECT_DIRECTIONAL,
    };
    use crate::irq::EmptyIrqHandler;
    use crate::memory::Memory;

    #[test]
    fn test_keys_from_str() {
        assert_eq!(JoypadKey::from_string_slice("Up"), Some(Up));
        assert_eq!(JoypadKey::from_string_slice("Down"), Some(Down));
        assert_eq!(JoypadKey::from_string_slice("Left"), Some(Left));
        assert_eq!(JoypadKey::from_string_slice("Right"), Some(Right));
        assert_eq!(JoypadKey::from_string_slice("Select"), Some(Select));
        assert_eq!(JoypadKey::from_string_slice("Start"), Some(Start));
        assert_eq!(JoypadKey::from_string_slice("A"), Some(A));
        assert_eq!(JoypadKey::from_string_slice("B"), Some(B));
    }

    #[test]
    fn test_keys_down_and_up() {
        let mut irq_handler = EmptyIrqHandler;
        let mut joypad = Joypad::default();
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x00);

        for key_str in JOYPAD_KEYS.iter() {
            joypad.key_down(
                &JoypadKey::from_string_slice(key_str).unwrap(),
                &mut irq_handler,
            );
        }
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_DIRECTIONAL);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x00);
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_BUTTON);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x00);

        for key_str in JOYPAD_KEYS.iter() {
            joypad.key_up(&JoypadKey::from_string_slice(key_str).unwrap());
        }
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x0F);
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_DIRECTIONAL);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x0F);
    }

    #[test]
    fn test_key_sequence() {
        let mut irq_handler = EmptyIrqHandler;
        let mut joypad = Joypad::default();
        // up+right+A
        joypad.key_down(&Up, &mut irq_handler);
        joypad.key_down(&Right, &mut irq_handler);
        joypad.key_down(&A, &mut irq_handler);
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_DIRECTIONAL);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0b1010);
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_BUTTON);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0b1110);
        // right+A+Start
        joypad.key_up(&Up);
        joypad.key_down(&Start, &mut irq_handler);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0b0110);
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_DIRECTIONAL);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0b1110);
        // none
        joypad.key_up(&Right);
        joypad.key_up(&A);
        joypad.key_up(&Start);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x0F);
        joypad.write_byte(JOYPAD_ADDRESS, JOYPAD_SELECT_BUTTON);
        assert_eq!(joypad.read_byte(JOYPAD_ADDRESS), 0x0F);
    }
}
