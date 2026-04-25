// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{Input, InputAction, InputEventContents, InputType, Keymap, DEFAULT_KEYMAP};
use config::{Config, ConfigError, FileFormat};
use crossterm::event::{
    Event, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent,
};
use std::collections::HashMap;

pub(super) fn create_keymap(
    mut keymap: Keymap,
    keymap_options: Vec<&str>,
) -> Result<Keymap, ConfigError> {
    if keymap_options.is_empty() {
        return Ok(keymap);
    }

    let mut builder = Config::builder();
    for ko in keymap_options {
        builder = builder.add_source(config::File::from_str(ko, FileFormat::Ini).required(false));
    }

    let config = builder.build()?;
    let inputs = config.try_deserialize::<HashMap<Input, InputAction>>()?;

    for (k, a) in inputs {
        match k.input {
            InputType::Key(key) => {
                let key_event = KeyEvent {
                    code: key.code,
                    modifiers: key.modifiers,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                };
                keymap.insert(
                    Event::Key(key_event),
                    InputEventContents {
                        input: k,
                        action: a,
                    },
                );
            }
            InputType::Mouse(mouse) => {
                let mouse_event = MouseEvent {
                    kind: mouse.action,
                    column: 0,
                    row: 0,
                    modifiers: KeyModifiers::empty(),
                };
                keymap.insert(
                    Event::Mouse(mouse_event),
                    InputEventContents {
                        input: k,
                        action: a,
                    },
                );
            }
        }
    }

    Ok(keymap)
}

pub fn generate_keymap(keymap_options: Vec<&str>) -> Result<Keymap, ConfigError> {
    let keymap = default_keymap();
    create_keymap(keymap, keymap_options)
}

pub fn default_keymap() -> Keymap {
    let default_keymap = DEFAULT_KEYMAP.to_vec();
    let keymap = HashMap::new();
    let result = create_keymap(keymap, default_keymap);
    match result {
        Ok(keymap) => keymap,
        Err(err) => {
            eprintln!("Failed to load default keymap: {err}");
            HashMap::new()
        }
    }
}
