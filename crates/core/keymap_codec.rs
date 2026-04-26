// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{Input, InputType, Key, Mouse};
use crate::errors::HwatchError;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{de, Deserialize, Serialize};

impl Input {
    pub fn to_str(&self) -> String {
        match &self.input {
            InputType::Key(key) => {
                let modifiers = key
                    .modifiers
                    .iter()
                    .filter_map(modifier_to_string)
                    .collect::<Vec<&str>>()
                    .join("-");
                let code = keycode_to_string(key.code).unwrap_or_else(|| "unknown".to_string());
                if modifiers.is_empty() {
                    code
                } else {
                    format!("{}-{}", modifiers, code)
                }
            }
            InputType::Mouse(mouse) => {
                let action = mouse_action_to_string(mouse.action);
                format!("mouse-{}", action)
            }
        }
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.input {
            InputType::Key(key) => key.serialize(serializer),
            InputType::Mouse(mouse) => mouse.serialize(serializer),
        }
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let modifiers = self
            .modifiers
            .iter()
            .filter_map(modifier_to_string)
            .collect::<Vec<&str>>()
            .join("-");
        let code = keycode_to_string(self.code)
            .ok_or(HwatchError::ConfigError)
            .map_err(S::Error::custom)?;
        let formatted = if modifiers.is_empty() {
            code
        } else {
            format!("{}-{}", modifiers, code)
        };
        serializer.serialize_str(&formatted)
    }
}

impl Serialize for Mouse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let formatted = format!("mouse-{}", mouse_action_to_string(self.action));
        serializer.serialize_str(&formatted)
    }
}

impl<'de> Deserialize<'de> for Input {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        let tokens = value.split('-').collect::<Vec<&str>>();
        let input = match tokens[0] {
            "mouse" => {
                let mouse =
                    Mouse::deserialize(de::value::StrDeserializer::<D::Error>::new(&value))?;
                Input {
                    input: InputType::Mouse(mouse),
                }
            }
            _ => {
                let key = Key::deserialize(de::value::StrDeserializer::<D::Error>::new(&value))?;
                Input {
                    input: InputType::Key(key),
                }
            }
        };

        Ok(input)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        let tokens = value.split('-').collect::<Vec<&str>>();

        let mut modifiers = KeyModifiers::empty();

        for modifier in tokens.iter().take(tokens.len() - 1) {
            match modifier.to_ascii_lowercase().as_ref() {
                "shift" => modifiers.insert(KeyModifiers::SHIFT),
                "ctrl" => modifiers.insert(KeyModifiers::CONTROL),
                "alt" => modifiers.insert(KeyModifiers::ALT),
                "super" => modifiers.insert(KeyModifiers::SUPER),
                "hyper" => modifiers.insert(KeyModifiers::HYPER),
                "meta" => modifiers.insert(KeyModifiers::META),
                _ => {}
            };
        }

        let last = tokens
            .last()
            .ok_or(HwatchError::ConfigError)
            .map_err(D::Error::custom)?;

        let code = match last.to_ascii_lowercase().as_ref() {
            "esc" => KeyCode::Esc,
            "enter" => KeyCode::Enter,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "backtab" => KeyCode::BackTab,
            "backspace" => KeyCode::Backspace,
            "del" => KeyCode::Delete,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "ins" => KeyCode::Insert,
            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),
            "space" => KeyCode::Char(' '),
            "plus" => KeyCode::Char('+'),
            "minus" => KeyCode::Char('-'),
            "hyphen" => KeyCode::Char('-'),
            "equal" => KeyCode::Char('='),
            "tab" => KeyCode::Tab,
            c if c.len() == 1 => {
                let ch = c
                    .chars()
                    .next()
                    .ok_or(HwatchError::ConfigError)
                    .map_err(D::Error::custom)?;
                KeyCode::Char(ch)
            }
            _ => {
                return Err(D::Error::custom(HwatchError::ConfigError));
            }
        };
        Ok(Key { code, modifiers })
    }
}

impl<'de> Deserialize<'de> for Mouse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        let tokens = value.split('-').collect::<Vec<&str>>();
        let last = tokens
            .last()
            .ok_or(HwatchError::ConfigError)
            .map_err(D::Error::custom)?;

        let action = match last.to_ascii_lowercase().as_ref() {
            "button_down_left" => MouseEventKind::Down(MouseButton::Left),
            "button_down_right" => MouseEventKind::Down(MouseButton::Right),
            "button_up_left" => MouseEventKind::Up(MouseButton::Left),
            "button_up_right" => MouseEventKind::Up(MouseButton::Right),
            "scroll_up" => MouseEventKind::ScrollUp,
            "scroll_down" => MouseEventKind::ScrollDown,
            "scroll_left" => MouseEventKind::ScrollLeft,
            "scroll_right" => MouseEventKind::ScrollRight,
            _ => {
                return Err(D::Error::custom(HwatchError::ConfigError));
            }
        };

        Ok(Mouse { action })
    }
}

impl From<MouseEvent> for Input {
    fn from(value: MouseEvent) -> Self {
        Self {
            input: InputType::Mouse(Mouse { action: value.kind }),
        }
    }
}

impl From<KeyEvent> for Input {
    fn from(value: KeyEvent) -> Self {
        Self {
            input: InputType::Key(Key {
                code: value.code,
                modifiers: value.modifiers,
            }),
        }
    }
}

impl From<KeyEvent> for Key {
    fn from(value: KeyEvent) -> Self {
        Self {
            code: value.code,
            modifiers: value.modifiers,
        }
    }
}

fn modifier_to_string<'a>(modifier: KeyModifiers) -> Option<&'a str> {
    match modifier {
        KeyModifiers::SHIFT => Some("shift"),
        KeyModifiers::CONTROL => Some("ctrl"),
        KeyModifiers::ALT => Some("alt"),
        KeyModifiers::SUPER => Some("super"),
        KeyModifiers::HYPER => Some("hyper"),
        KeyModifiers::META => Some("meta"),
        _ => None,
    }
}

fn keycode_to_string(code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc => Some("esc".to_owned()),
        KeyCode::Enter => Some("enter".to_owned()),
        KeyCode::Left => Some("left".to_owned()),
        KeyCode::Right => Some("right".to_owned()),
        KeyCode::Up => Some("up".to_owned()),
        KeyCode::Down => Some("down".to_owned()),
        KeyCode::Home => Some("home".to_owned()),
        KeyCode::End => Some("end".to_owned()),
        KeyCode::PageUp => Some("pageup".to_owned()),
        KeyCode::PageDown => Some("pagedown".to_owned()),
        KeyCode::BackTab => Some("backtab".to_owned()),
        KeyCode::Backspace => Some("backspace".to_owned()),
        KeyCode::Delete => Some("delete".to_owned()),
        KeyCode::Insert => Some("insert".to_owned()),
        KeyCode::F(1) => Some("f1".to_owned()),
        KeyCode::F(2) => Some("f2".to_owned()),
        KeyCode::F(3) => Some("f3".to_owned()),
        KeyCode::F(4) => Some("f4".to_owned()),
        KeyCode::F(5) => Some("f5".to_owned()),
        KeyCode::F(6) => Some("f6".to_owned()),
        KeyCode::F(7) => Some("f7".to_owned()),
        KeyCode::F(8) => Some("f8".to_owned()),
        KeyCode::F(9) => Some("f9".to_owned()),
        KeyCode::F(10) => Some("f10".to_owned()),
        KeyCode::F(11) => Some("f11".to_owned()),
        KeyCode::F(12) => Some("f12".to_owned()),
        KeyCode::Char(' ') => Some("space".to_owned()),
        KeyCode::Tab => Some("tab".to_owned()),
        KeyCode::Char(c) => Some(String::from(c)),
        _ => None,
    }
}

fn mouse_action_to_string(action: MouseEventKind) -> &'static str {
    match action {
        MouseEventKind::Down(MouseButton::Left) => "button_down_left",
        MouseEventKind::Down(MouseButton::Right) => "button_down_right",
        MouseEventKind::Up(MouseButton::Left) => "button_up_left",
        MouseEventKind::Up(MouseButton::Right) => "button_up_right",
        MouseEventKind::ScrollUp => "scroll_up",
        MouseEventKind::ScrollDown => "scroll_down",
        MouseEventKind::ScrollLeft => "scroll_left",
        MouseEventKind::ScrollRight => "scroll_right",
        _ => "other",
    }
}
