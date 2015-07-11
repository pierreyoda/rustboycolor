#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeypadKey {
    Up,
    Down,
    Right,
    Left,
    A,
    B,
    Select,
    Start,
}

impl KeypadKey {
    /// Build and return a KeypadKey corresponding to the given symbol, if possible.
    pub fn from_str(symbol: &str) -> Option<KeypadKey> {
        match symbol {
            "Up"     => Some(KeypadKey::Up),
            "Down"   => Some(KeypadKey::Down),
            "Right"  => Some(KeypadKey::Right),
            "Left"   => Some(KeypadKey::Left),
            "A"      => Some(KeypadKey::A),
            "B"      => Some(KeypadKey::B),
            "Select" => Some(KeypadKey::Select),
            "Start"  => Some(KeypadKey::Start),
            _        => None,
        }
    }
}
