use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use std::hash::Hash;

use toml;

use rustboylib::joypad::{JoypadKey, JOYPAD_KEYS};
use self::KeyboardBinding::*;

/// Enumerates the supported keyboard bindings for the virtual joypad.
#[derive(Clone, PartialEq)]
pub enum KeyboardBinding {
    /// QWERTY binding (the default one).
    QWERTY,
    /// AZERTY binding.
    AZERTY,
    /// The keyboard binding is to be loaded from the given configuration file.
    /// If this fails, revert to the default binding ('QWERTY').
    FromConfigFile(String),
}

impl fmt::Debug for KeyboardBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QWERTY => write!(f, "QWERTY"),
            AZERTY => write!(f, "AZERTY"),
            FromConfigFile(ref file) => write!(f, "in configuration file \"{}\"", file),
        }
    }
}

/// Get the 'HashMap' translating between the emulator's 'JoypadKey' and the
/// backend's keycode type and corresponding to the given 'KeyboardBinding'.
/// The given 'HashMap' provides the keycode corresponding to the associated
/// symbol ; if it is not specified, the function fails.
///
/// The following (backend-agnostic) symbols should be supported :
/// "Numpad{0..9}"
/// "NumpadMinus" / "NumpadPlus" / "NumpadDivide" / "NumpadMultiply"
/// "{A..Z}"
/// "F{0..12}"
/// "Up" / "Down" / "Left" / "Right"
pub fn get_key_bindings<Key>(binding: KeyboardBinding,
                             symbol_backend_key_hm: HashMap<String, Key>)
                             -> Result<HashMap<Key, JoypadKey>, String>
    where Key: Hash + Eq + Copy
{
    let mut hm = HashMap::new();

    let keyboard_control_hm = try!(build_keyboard_control_hm(binding));
    for (symbol, control) in &keyboard_control_hm {
        let key = match symbol_backend_key_hm.get(symbol) {
            Some(ref k) => *(k.clone()),
            None => return Err(format!("backend does not support key \"{}\"", symbol)),
        };
        hm.insert(key, *control);
    }

    Ok(hm)
}

fn build_keyboard_control_hm(binding: KeyboardBinding) -> Result<HashMap<String, JoypadKey>, String> {

    match binding {
        QWERTY => {
            let mut hm = HashMap::new();
            hm.insert("W".into(), JoypadKey::Up);
            hm.insert("S".into(), JoypadKey::Down);
            hm.insert("A".into(), JoypadKey::Left);
            hm.insert("D".into(), JoypadKey::Right);
            hm.insert("Z".into(), JoypadKey::Select);
            hm.insert("C".into(), JoypadKey::Start);
            hm.insert("G".into(), JoypadKey::A);
            hm.insert("Y".into(), JoypadKey::B);
            assert_eq!(hm.len(), 8);
            Ok(hm)
        }
        AZERTY => {
            let mut hm = HashMap::new();
            hm.insert("W".into(), JoypadKey::Up);
            hm.insert("S".into(), JoypadKey::Down);
            hm.insert("A".into(), JoypadKey::Left);
            hm.insert("D".into(), JoypadKey::Right);
            hm.insert("Z".into(), JoypadKey::Select);
            hm.insert("C".into(), JoypadKey::Start);
            hm.insert("G".into(), JoypadKey::A);
            hm.insert("Y".into(), JoypadKey::B);
            assert_eq!(hm.len(), 8);
            Ok(hm)
        }
        FromConfigFile(ref config_file) => {
            let filepath = Path::new(&config_file[..]);
            let mut file_content = String::new();
            try!(File::open(filepath)
                     .and_then(|mut f| f.read_to_string(&mut file_content))
                     .map_err(|_| {
                         format!("could not load the input config file : {}",
                                 filepath.display())
                     }));
            keyboard_hm_from_config(&file_content[..], format!("{}", filepath.display()))
        }
    }
}

fn keyboard_hm_from_config<'a>(config_str: &'a str,
                               config_file: String)
                               -> Result<HashMap<String, JoypadKey>, String> {
    let mut hm = HashMap::new();

    let mut parser = toml::Parser::new(config_str);
    let table: toml::Value = match parser.parse() {
        Some(t) => toml::Value::Table(t),
        None => return Err(format!("parsing error in input config : {:?}", parser.errors)),
    };
    let keyboard_input = match table.lookup("input.keyboard") {
        Some(value) => value,
        None => {
            warn!(concat!("input config file \"{}\" does not specify",
                          " keyboard input, reverting to QWERTY."),
                  config_file);
            return build_keyboard_control_hm(QWERTY);
        }
    };

    for key in JOYPAD_KEYS.iter() {
        let key_symbol = match keyboard_input.lookup(key) {
            Some(value) => {
                match *value {
                    toml::Value::String(ref s) => &s[..],
                    _ => {
                        return Err(format!("key \"{}\" does not have a String value in input \
                                            config",
                                           key))
                    }
                }
            }
            None => return Err(format!("no key specified for \"{}\" in input config", key)),
        };
        if hm.insert(key_symbol.into(), JoypadKey::from_str(key).unwrap()).is_some() {
            warn!("input config file \"{}\" binds key \"{}\" more than once, earlier \
                   occurences will be erased",
                  config_file,
                  key_symbol);
            // N.B. : the order of priority is the one defined in JOYPAD_KEYS
            // which may not be naturally the one written in the configuration file
        }
    }
    if hm.len() == JOYPAD_KEYS.len() {
        Ok(hm)
    } else {
        Err(format!("missing joypad key(s) in input config file \"{}\"",
                    config_file))
    }
}

#[cfg(test)]
mod test {
    use rustboylib::joypad::JoypadKey;

    #[test]
    fn test_keyboard_hm_from_config() {
        let config = r#"
        [input]
            [input.keyboard]
            Up     = "Up"
            Down   = "Down"
            Left   = "Left"
            Right  = "Right"
            Select = "Numpad1"
            Start  = "Numpad3"
            A      = "E"
            B      = "T"
        "#;
        let r = super::keyboard_hm_from_config(config, "*test*".into());
        assert!(r.is_ok());
        let keys_hm = r.unwrap();
        assert_eq!(*keys_hm.get("Up".into()).unwrap(), JoypadKey::Up);
        assert_eq!(*keys_hm.get("Down".into()).unwrap(), JoypadKey::Down);
        assert_eq!(*keys_hm.get("Left".into()).unwrap(), JoypadKey::Left);
        assert_eq!(*keys_hm.get("Right".into()).unwrap(), JoypadKey::Right);
        assert_eq!(*keys_hm.get("Numpad1".into()).unwrap(), JoypadKey::Select);
        assert_eq!(*keys_hm.get("Numpad3".into()).unwrap(), JoypadKey::Start);
        assert_eq!(*keys_hm.get("E".into()).unwrap(), JoypadKey::A);
        assert_eq!(*keys_hm.get("T".into()).unwrap(), JoypadKey::B);
    }
}
