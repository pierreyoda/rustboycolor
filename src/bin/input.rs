use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use std::hash::Hash;

use toml;

use rustboylib::keypad::KeypadKey;
use self::KeyboardBinding::*;

/// Enumerates the supported keyboard bindings for the virtual keypad.
/// TODO : add a Custom(...file?...) type, loaded from a file
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

/// Get the 'HashMap' translating between the emulator's 'KeypadKey' and the
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
    -> Result<HashMap<Key, KeypadKey>, String>
    where Key: Hash+Eq+Copy {
    let mut hm = HashMap::new();

    let keyboard_control_hm = try!(build_keyboard_control_hm(binding));
    for (symbol, control) in &keyboard_control_hm {
        let key = match symbol_backend_key_hm.get(symbol) {
            Some(ref k) => *(k.clone()),
            None => return Err(format!("backend does not support key \"{}\"",
                                       symbol)),
        };
        hm.insert(key, *control);
    }

    Ok(hm)
}


/// All the keypad keys an input configuration must bind.
static KEYPAD_KEYS: &'static [&'static str] = &["Up", "Down", "Right", "Left",
    "A", "B", "Select", "Start"];

fn build_keyboard_control_hm(binding: KeyboardBinding) ->
    Result<HashMap<String, KeypadKey>, String> {

    match binding {
        QWERTY => {
            let mut hm = HashMap::new();
            hm.insert("W".into(), KeypadKey::Up);
            hm.insert("S".into(), KeypadKey::Down);
            hm.insert("A".into(), KeypadKey::Left);
            hm.insert("D".into(), KeypadKey::Right);
            hm.insert("Z".into(), KeypadKey::Select);
            hm.insert("C".into(), KeypadKey::Start);
            hm.insert("G".into(), KeypadKey::A);
            hm.insert("Y".into(), KeypadKey::B);
            assert_eq!(hm.len(), 8);
            Ok(hm)
        }
        AZERTY => {
            let mut hm = HashMap::new();
            hm.insert("W".into(), KeypadKey::Up);
            hm.insert("S".into(), KeypadKey::Down);
            hm.insert("A".into(), KeypadKey::Left);
            hm.insert("D".into(), KeypadKey::Right);
            hm.insert("Z".into(), KeypadKey::Select);
            hm.insert("C".into(), KeypadKey::Start);
            hm.insert("G".into(), KeypadKey::A);
            hm.insert("Y".into(), KeypadKey::B);
            assert_eq!(hm.len(), 8);
            Ok(hm)
        },
        FromConfigFile(ref config_file) => {
            let filepath = Path::new(&config_file[..]);
            let mut file_content = String::new();
            try!(File::open(filepath).and_then(|mut f| f.read_to_string(&mut file_content))
                .map_err(|_| format!("could not load the config file : {}",
                                     filepath.display())));
            keyboard_hm_from_config(&file_content[..],
                                    format!("{}", filepath.display()))
        },
    }
}

fn keyboard_hm_from_config<'a>(config_str: &'a str, config_file: String)
    -> Result<HashMap<String, KeypadKey>, String> {
    let mut hm = HashMap::new();

    let mut parser = toml::Parser::new(config_str);
    let table: toml::Value = match parser.parse() {
        Some(t) => toml::Value::Table(t),
        None => return Err(format!("parsing error in config : {:?}", parser.errors)),
    };
    let keyboard_input = match table.lookup("input.keyboard") {
        Some(value) => value,
        None => {
            warn!(concat!("config file \"{}\" does not specify",
                          " keyboard input, reverting to QWERTY."),
                  config_file);
            return build_keyboard_control_hm(QWERTY);
        },
    };

    for key in KEYPAD_KEYS {
        let key_symbol = match keyboard_input.lookup(key) {
            Some(value) => match *value {
                toml::Value::String(ref s) => &s[..],
                _ => return Err(
                    format!("key \"{}\" does not have a String value in config", key)),
            },
            None => return Err(format!("no key specified for \"{}\" in config", key)),
        };
        hm.insert(key_symbol.into(), KeypadKey::from_str(key).unwrap());
    }
    Ok(hm)
}

#[cfg(test)]
mod test {
    use rustboylib::keypad::KeypadKey;

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
        for (k, v) in &keys_hm {
            println!("{} = {:?}", k, v);
        }
        assert_eq!(*keys_hm.get("Up".into()).unwrap(), KeypadKey::Up);
        assert_eq!(*keys_hm.get("Down".into()).unwrap(), KeypadKey::Down);
        assert_eq!(*keys_hm.get("Left".into()).unwrap(), KeypadKey::Left);
        assert_eq!(*keys_hm.get("Right".into()).unwrap(), KeypadKey::Right);
        assert_eq!(*keys_hm.get("Numpad1".into()).unwrap(), KeypadKey::Select);
        assert_eq!(*keys_hm.get("Numpad3".into()).unwrap(), KeypadKey::Start);
        assert_eq!(*keys_hm.get("E".into()).unwrap(), KeypadKey::A);
        assert_eq!(*keys_hm.get("T".into()).unwrap(), KeypadKey::B);
    }
}
