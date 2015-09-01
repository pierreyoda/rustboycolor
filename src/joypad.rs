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
    pub fn from_str(symbol: &str) -> Option<JoypadKey> {
        match symbol {
            "Up"     => Some(JoypadKey::Up),
            "Down"   => Some(JoypadKey::Down),
            "Right"  => Some(JoypadKey::Right),
            "Left"   => Some(JoypadKey::Left),
            "A"      => Some(JoypadKey::A),
            "B"      => Some(JoypadKey::B),
            "Select" => Some(JoypadKey::Select),
            "Start"  => Some(JoypadKey::Start),
            _        => None,
        }
    }
}
