use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use toml;

use self::KeyboardBinding::*;
use rustboylib::joypad::{JOYPAD_KEYS, JoypadKey};

/// Enumerates the supported keyboard bindings for the virtual joypad.
#[derive(Clone, PartialEq)]
pub enum KeyboardBinding {
    /// QWERTY binding (the default one).
    QWERTY,
    /// AZERTY binding.
    AZERTY,
    /// The keyboard binding is to be loaded from the given configuration file.
    /// If this fails, revert to the default binding ('QWERTY').
    FromConfigFile(PathBuf),
}

impl fmt::Debug for KeyboardBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            QWERTY => write!(f, "QWERTY"),
            AZERTY => write!(f, "AZERTY"),
            FromConfigFile(ref filename) => {
                write!(f, "in configuration file \"{}\"", filename.display())
            }
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
pub fn get_key_bindings<Key>(
    binding: &KeyboardBinding,
    symbol_backend_key_hm: &HashMap<String, Key>,
) -> Result<HashMap<Key, JoypadKey>, String>
where
    Key: Hash + Eq + Copy,
{
    let mut hm = HashMap::new();

    let keyboard_control_hm = build_keyboard_control_hm(&binding)?;
    for (symbol, control) in &keyboard_control_hm {
        let key = match symbol_backend_key_hm.get(symbol) {
            Some(k) => *k,
            None => return Err(format!("backend does not support key \"{}\"", symbol)),
        };
        hm.insert(key, *control);
    }

    Ok(hm)
}

fn build_keyboard_control_hm(
    binding: &KeyboardBinding,
) -> Result<HashMap<String, JoypadKey>, String> {
    match *binding {
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
            hm.insert("Z".into(), JoypadKey::Up);
            hm.insert("S".into(), JoypadKey::Down);
            hm.insert("Q".into(), JoypadKey::Left);
            hm.insert("D".into(), JoypadKey::Right);
            hm.insert("W".into(), JoypadKey::Select);
            hm.insert("C".into(), JoypadKey::Start);
            hm.insert("G".into(), JoypadKey::A);
            hm.insert("Y".into(), JoypadKey::B);
            assert_eq!(hm.len(), 8);
            Ok(hm)
        }
        FromConfigFile(ref config_file) => {
            let filepath = Path::new(config_file);
            let mut file_content = String::new();
            File::open(filepath)
                .and_then(|mut f| f.read_to_string(&mut file_content))
                .map_err(|_| {
                    format!(
                        "could not load the input config file : {}",
                        filepath.display()
                    )
                })?;
            keyboard_hm_from_config(&file_content[..], &format!("{}", filepath.display()))
        }
    }
}

fn keyboard_hm_from_config(
    config_str: &str,
    config_file: &str,
) -> Result<HashMap<String, JoypadKey>, String> {
    let mut hm = HashMap::new();
    let table = match config_str.parse::<toml::Table>() {
        Ok(table) => table,
        Err(err) => {
            return Err(format!(
                "parsing error in input config file \"{}\" : {}",
                config_file, err
            ));
        }
    };
    let input = match table.get("input") {
        Some(value) => value.as_table().expect("no input section specified"),
        None => {
            warn!(
                concat!(
                    "input config file \"{}\" does not specify",
                    "any input configuration, reverting to QWERTY."
                ),
                config_file
            );
            return build_keyboard_control_hm(&QWERTY);
        }
    };
    let keyboard_input = match input.get("keyboard") {
        Some(value) => value
            .as_table()
            .expect("no input.keyboard subsection specified"),
        None => {
            warn!(
                concat!(
                    "input config file \"{}\" does not specify",
                    " keyboard input, reverting to QWERTY."
                ),
                config_file
            );
            return build_keyboard_control_hm(&QWERTY);
        }
    };

    for key in &JOYPAD_KEYS {
        let key_symbol = match keyboard_input.get(&key.to_string()) {
            Some(value) => match *value {
                toml::Value::String(ref s) => &s[..],
                _ => {
                    return Err(format!(
                        "key \"{}\" does not have a String value in input \
                         config",
                        key
                    ));
                }
            },
            None => {
                warn!(
                    "no key specified for \"{}\" in input config, reverting to QWERTY",
                    key
                );
                return build_keyboard_control_hm(&QWERTY);
            }
        };
        if hm
            .insert(
                key_symbol.into(),
                JoypadKey::from_string_slice(key).unwrap(),
            )
            .is_some()
        {
            warn!(
                "input config file \"{}\" binds key \"{}\" more than once, earlier \
                 occurrences will be erased",
                config_file, key_symbol
            );
            // N.B. : the order of priority is the one defined in JOYPAD_KEYS
            // which may not be naturally the one written in the configuration file
        }
    }
    if hm.len() == JOYPAD_KEYS.len() {
        Ok(hm)
    } else {
        Err(format!(
            "missing joypad key(s) in input config file \"{}\"",
            config_file
        ))
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
        assert_eq!(keys_hm.get(&"Up".to_string()).unwrap(), &JoypadKey::Up);
        assert_eq!(keys_hm.get(&"Down".to_string()).unwrap(), &JoypadKey::Down);
        assert_eq!(keys_hm.get(&"Left".to_string()).unwrap(), &JoypadKey::Left);
        assert_eq!(
            keys_hm.get(&"Right".to_string()).unwrap(),
            &JoypadKey::Right
        );
        assert_eq!(
            keys_hm.get(&"Numpad1".to_string()).unwrap(),
            &JoypadKey::Select
        );
        assert_eq!(
            keys_hm.get(&"Numpad3".to_string()).unwrap(),
            &JoypadKey::Start
        );
        assert_eq!(keys_hm.get(&"E".to_string()).unwrap(), &JoypadKey::A);
        assert_eq!(keys_hm.get(&"T".to_string()).unwrap(), &JoypadKey::B);
    }
}
