// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use config::{Config, ConfigError, FileFormat};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{de, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

use crate::errors::HwatchError;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Mouse {
    action: MouseEventKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
enum InputType {
    Key(Key),
    Mouse(Mouse),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Input {
    input: InputType,
}

impl Input {
    pub fn to_str(&self) -> String {
        let result = match &self.input {
            // keyboard
            InputType::Key(key) => {
                let modifiers = key
                    .modifiers
                    .iter()
                    .filter_map(modifier_to_string)
                    .collect::<Vec<&str>>()
                    .join("-");
                let code = keycode_to_string(key.code).unwrap();
                if modifiers.is_empty() {
                    code
                } else {
                    format!("{}-{}", modifiers, code)
                }
            }

            // mouse
            InputType::Mouse(mouse) => {
                let action = match mouse.action {
                    // MouseButton
                    MouseEventKind::Down(MouseButton::Left) => "button_down_left",
                    MouseEventKind::Down(MouseButton::Right) => "button_down_right",
                    MouseEventKind::Up(MouseButton::Left) => "button_up_left",
                    MouseEventKind::Up(MouseButton::Right) => "button_up_right",

                    // MouseScroll
                    MouseEventKind::ScrollUp => "scroll_up",
                    MouseEventKind::ScrollDown => "scroll_down",
                    MouseEventKind::ScrollLeft => "scroll_left",
                    MouseEventKind::ScrollRight => "scroll_right",

                    _ => "other",
                };

                format!("mouse-{}", action)
            }
        };

        return result;
    }
}

// NOTE: shfit + alt + keyは動くので、とりあえず横スクロールの移動はこれにしとくか？
//       もしくは、適当にHome/End + altにでもしとく？？？押しにくいので、デフォルトは↑のがいいけど

const DEFAULT_KEYMAP: [&str; 45] = [
    "up=up",                                    // Up
    "down=down",                                // Down
    "pageup=page_up",                           // PageUp
    "pagedown=page_down",                       // PageDown
    "home=move_top",                            // MoveTop: Home
    "end=move_end",                             // MoveEnd: End
    "tab=toggle_forcus",                        // ToggleForcus: Tab
    "left=forcus_watch_pane",                   // ForcusWatchPane: Left
    "right=forcus_history_pane",                // ForcusHistoryPane: Right
    "alt-left=scroll_left",                     // Watch window scrll left: Alt + Left
    "shift-alt-left=scroll_horizontal_home",    //
    "alt-right=scroll_right",                   // Watch window scrll right: Alt + Right
    "shift-alt-right=scroll_horizontal_end",    //
    "q=quit",                                   // Quit: q
    "esc=reset",                                // Reset: ESC
    "ctrl-c=cancel",                            // Cancel: Ctrl + c
    "h=help",                                   // Help: h
    "b=toggle_border_with_scroll_bar",          // Toggle Border: b
    "c=toggle_color",                           // Toggle Color: c
    "n=toggle_line_number",                     // Toggle Line Number: n
    "r=toggle_reverse",                         // Toggle Reverse: r
    "m=toggle_mouse_support",                   // Toggle Mouse Support: m
    "t=toggle_view_pane_ui",                    // Toggle View Pane UI: t
    "backspace=toggle_view_history_pane",       // Toggle View History Pane: Backspace
    "d=toggle_diff_mode",                       // Toggle Diff Mode: d
    "0=set_diff_mode_plane",                    // Set Diff Mode Plane: 0
    "1=set_diff_mode_watch",                    // Set Diff Mode Watch: 1
    "2=set_diff_mode_line",                     // Set Diff Mode Line: 2
    "3=set_diff_mode_word",                     // Set Diff Mode Word: 3
    "shift-o=set_diff_only",                    // Set Diff Only: Shift + o
    "o=toggle_output_mode",                     // Toggle Output Mode: o
    "f3=set_output_mode_output",                // Set Output Mode Output: F3
    "f1=set_output_mode_stdout",                // Set Output Mode Stdout: F1
    "f2=set_output_mode_stderr",                // Set Output Mode Stderr: F2
    "w=toggle_wrap_mode",                       // Toggle Wrap mode: w
    "ctrl-n=next_keyword",                      //
    "ctrl-p=prev_keyword",                      //
    "shift-s=togge_history_summary",            //
    "plus=interval_plus",                       // Interval Plus: +
    "minus=interval_minus",                     // Interval Minus: -
    "/=change_filter_mode",                     // Change Filter Mode: /
    "*=change_regex_filter_mode",               // Change Regex Filter Mode: *
    "mouse-scroll_up=mouse_scroll_up",          // Mouse Scroll Up: Mouse Scroll Up
    "mouse-scroll_down=mouse_scroll_down",      // Mouse Scroll Down: Mouse Scroll Down
    "mouse-button_down_left=mouse_button_left", // Mouse Button Left: Mouse Button Left
];

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
        let action = match self.action {
            // MouseButton
            MouseEventKind::Down(MouseButton::Left) => "button_down_left",
            MouseEventKind::Down(MouseButton::Right) => "button_down_right",
            MouseEventKind::Up(MouseButton::Left) => "button_up_left",
            MouseEventKind::Up(MouseButton::Right) => "button_up_right",

            // MouseScroll
            MouseEventKind::ScrollUp => "scroll_up",
            MouseEventKind::ScrollDown => "scroll_down",
            MouseEventKind::ScrollLeft => "scroll_left",
            MouseEventKind::ScrollRight => "scroll_right",

            _ => "other",
        };

        let formatted = format!("mouse-{}", action);
        serializer.serialize_str(&formatted)
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
            c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct InputEventContents {
    pub input: Input,
    pub action: InputAction,
}

// pub type Keymap = HashMap<Event, InputAction>;
pub type Keymap = HashMap<Event, InputEventContents>;

pub fn generate_keymap(keymap_options: Vec<&str>) -> Result<Keymap, ConfigError> {
    let keymap = default_keymap();
    let result = create_keymap(keymap, keymap_options);
    return result;
}

///
fn create_keymap(mut keymap: Keymap, keymap_options: Vec<&str>) -> Result<Keymap, ConfigError> {
    if keymap_options.len() == 0 {
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

pub fn default_keymap() -> Keymap {
    let default_keymap = DEFAULT_KEYMAP.to_vec();
    let keymap = HashMap::new();
    let result = create_keymap(keymap, default_keymap);
    return result.unwrap();
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputAction {
    // None
    // ==========
    #[serde(rename = "none")]
    None,

    // Up
    // ==========
    #[serde(rename = "up")]
    Up,
    #[serde(rename = "watch_pane_up")]
    WatchPaneUp,
    #[serde(rename = "history_pane_up")]
    HistoryPaneUp,

    // Down
    // ==========
    #[serde(rename = "down")]
    Down,
    #[serde(rename = "watch_pane_down")]
    WatchPaneDown,
    #[serde(rename = "history_pane_down")]
    HistoryPaneDown,

    // Scroll right
    // ==========
    #[serde(rename = "scroll_right")]
    ScrollRight,
    #[serde(rename = "scroll_horizontal_home")]
    ScrollHorizontalHome,

    // Scroll left
    // ==========
    #[serde(rename = "scroll_left")]
    ScrollLeft,
    #[serde(rename = "scroll_horizontal_end")]
    ScrollHorizontalEnd,

    // PageUp
    // ==========
    #[serde(rename = "page_up")]
    PageUp,
    #[serde(rename = "watch_pane_page_up")]
    WatchPanePageUp,
    #[serde(rename = "history_pane_page_up")]
    HistoryPanePageUp,

    // PageDown
    // ==========
    #[serde(rename = "page_down")]
    PageDown,
    #[serde(rename = "watch_pane_page_down")]
    WatchPanePageDown,
    #[serde(rename = "history_pane_page_down")]
    HistoryPanePageDown,

    // MoveTop
    // ==========
    #[serde(rename = "move_top")]
    MoveTop,
    #[serde(rename = "watch_pane_move_top")]
    WatchPaneMoveTop,
    #[serde(rename = "history_pane_move_top")]
    HistoryPaneMoveTop,

    // MoveEnd
    // ==========
    #[serde(rename = "move_end")]
    MoveEnd,
    #[serde(rename = "watch_pane_move_end")]
    WatchPaneMoveEnd,
    #[serde(rename = "history_pane_move_end")]
    HistoryPaneMoveEnd,

    // Forcus
    // ==========
    #[serde(rename = "toggle_forcus")]
    ToggleForcus,
    #[serde(rename = "forcus_watch_pane")]
    ForcusWatchPane,
    #[serde(rename = "forcus_history_pane")]
    ForcusHistoryPane,

    // quit
    // ==========
    #[serde(rename = "quit")]
    Quit,

    // reset
    // ==========
    #[serde(rename = "reset")]
    Reset,

    // Cancel
    // ==========
    #[serde(rename = "cancel")]
    Cancel,
    #[serde(rename = "force_cancel")]
    ForceCancel,

    // help
    // ==========
    #[serde(rename = "help")]
    Help,

    // Color
    // ==========
    #[serde(rename = "toggle_color")]
    ToggleColor,

    // LineNumber
    // ==========
    #[serde(rename = "toggle_line_number")]
    ToggleLineNumber,

    // Reverse
    // ==========
    #[serde(rename = "toggle_reverse")]
    ToggleReverse,

    // Mouse Support
    // ==========
    #[serde(rename = "toggle_mouse_support")]
    ToggleMouseSupport,

    // Toggle View Pane UI
    // ==========
    #[serde(rename = "toggle_view_pane_ui")]
    ToggleViewPaneUI,
    #[serde(rename = "toggle_view_header_pane")]
    ToggleViewHeaderPane,
    #[serde(rename = "toggle_view_history_pane")]
    ToggleViewHistoryPane,

    // Border
    // ==========
    #[serde(rename = "toggle_border")]
    ToggleBorder,
    #[serde(rename = "toggle_scroll_bar")]
    ToggleScrollBar,
    #[serde(rename = "toggle_border_with_scroll_bar")]
    ToggleBorderWithScrollBar,

    // Diff Mode
    // ==========
    #[serde(rename = "toggle_diff_mode")]
    ToggleDiffMode,
    #[serde(rename = "set_diff_mode_plane")]
    SetDiffModePlane,
    #[serde(rename = "set_diff_mode_watch")]
    SetDiffModeWatch,
    #[serde(rename = "set_diff_mode_line")]
    SetDiffModeLine,
    #[serde(rename = "set_diff_mode_word")]
    SetDiffModeWord,
    #[serde(rename = "set_diff_only")]
    SetDiffOnly,

    // Output Mode
    // ==========
    #[serde(rename = "toggle_output_mode")]
    ToggleOutputMode,
    #[serde(rename = "set_output_mode_output")]
    SetOutputModeOutput,
    #[serde(rename = "set_output_mode_stdout")]
    SetOutputModeStdout,
    #[serde(rename = "set_output_mode_stderr")]
    SetOutputModeStderr,

    // Toggle Wrap
    // ==========
    #[serde(rename = "toggle_wrap_mode")]
    ToggleWrapMode,

    // Keyword search
    // ==========
    #[serde(rename = "next_keyword")]
    NextKeyword,
    #[serde(rename = "prev_keyword")]
    PrevKeyword,

    // HistorySummary
    #[serde(rename = "togge_history_summary")]
    ToggleHistorySummary,

    // Interval
    // ==========
    #[serde(rename = "interval_plus")]
    IntervalPlus,
    #[serde(rename = "interval_minus")]
    IntervalMinus,

    // Command/Filter
    // ==========
    #[serde(rename = "change_filter_mode")]
    ChangeFilterMode,
    #[serde(rename = "change_regex_filter_mode")]
    ChangeRegexFilterMode,

    // Mouse
    // ==========
    #[serde(rename = "mouse_scroll_up")]
    MouseScrollUp,
    #[serde(rename = "mouse_scroll_down")]
    MouseScrollDown,
    #[serde(rename = "mouse_button_left")]
    MouseButtonLeft,
    #[serde(rename = "mouse_button_right")]
    MouseButtonRight,
    #[serde(rename = "mouse_move_left")]
    MouseMoveLeft,
    #[serde(rename = "mouse_move_right")]
    MouseMoveRight,
    #[serde(rename = "mouse_move_up")]
    MouseMoveUp,
    #[serde(rename = "mouse_move_down")]
    MouseMoveDown,
}

pub fn get_input_action_description(input_action: InputAction) -> String {
    match input_action {
        // None
        InputAction::None => "No action".to_string(),

        // Up
        InputAction::Up => "Move up".to_string(),
        InputAction::WatchPaneUp => "Move up in watch pane".to_string(),
        InputAction::HistoryPaneUp => "Move up in history pane".to_string(),

        // Down
        InputAction::Down => "Move down".to_string(),
        InputAction::WatchPaneDown => "Move down in watch pane".to_string(),
        InputAction::HistoryPaneDown => "Move down in history pane".to_string(),

        // Shift Right
        InputAction::ScrollRight => "Move Right".to_string(),
        InputAction::ScrollHorizontalEnd => "Move Right end".to_string(),

        // Shift Left
        InputAction::ScrollLeft => "Move Left".to_string(),
        InputAction::ScrollHorizontalHome => "Move Left home".to_string(),

        // PageUp
        InputAction::PageUp => "Move page up".to_string(),
        InputAction::WatchPanePageUp => "Move page up in watch pane".to_string(),
        InputAction::HistoryPanePageUp => "Move page up in history pane".to_string(),

        // PageDown
        InputAction::PageDown => "Move page down".to_string(),
        InputAction::WatchPanePageDown => "Move page down in watch pane".to_string(),
        InputAction::HistoryPanePageDown => "Move page down in history pane".to_string(),

        // MoveTop
        InputAction::MoveTop => "Move top".to_string(),
        InputAction::WatchPaneMoveTop => "Move top in watch pane".to_string(),
        InputAction::HistoryPaneMoveTop => "Move top in history pane".to_string(),

        // MoveEnd
        InputAction::MoveEnd => "Move end".to_string(),
        InputAction::WatchPaneMoveEnd => "Move end in watch pane".to_string(),
        InputAction::HistoryPaneMoveEnd => "Move end in history pane".to_string(),

        // Forcus
        InputAction::ToggleForcus => "Toggle forcus window".to_string(),
        InputAction::ForcusWatchPane => "Forcus watch pane".to_string(),
        InputAction::ForcusHistoryPane => "Forcus history pane".to_string(),

        // Quit
        InputAction::Quit => "Quit hwatch".to_string(),

        // Reset
        InputAction::Reset => "filter reset".to_string(),

        // Cancel
        InputAction::Cancel => "Cancel".to_string(),
        InputAction::ForceCancel => "Cancel without displaying the exit dialog".to_string(),

        // Help
        InputAction::Help => "Show and hide help window".to_string(),

        // Color
        InputAction::ToggleColor => "Toggle enable/disable ANSI Color".to_string(),

        // LineNumber
        InputAction::ToggleLineNumber => "Toggle enable/disable Line Number".to_string(),

        // Reverse
        InputAction::ToggleReverse => "Toggle enable/disable text reverse".to_string(),

        // Mouse Support
        InputAction::ToggleMouseSupport => "Toggle enable/disable mouse support".to_string(),

        // Toggle View Pane UI
        InputAction::ToggleViewPaneUI => "Toggle view header/history pane".to_string(),
        InputAction::ToggleViewHeaderPane => "Toggle view header pane".to_string(),
        InputAction::ToggleViewHistoryPane => "Toggle view history pane".to_string(),

        // Border
        InputAction::ToggleBorder => "Toggle enable/disable border".to_string(),
        InputAction::ToggleScrollBar => "Toggle enable/disable scroll bar".to_string(),
        InputAction::ToggleBorderWithScrollBar => {
            "Toggle enable/disable border and scroll bar".to_string()
        }

        // Diff Mode
        InputAction::ToggleDiffMode => "Toggle diff mode".to_string(),
        InputAction::SetDiffModePlane => "Set diff mode plane".to_string(),
        InputAction::SetDiffModeWatch => "Set diff mode watch".to_string(),
        InputAction::SetDiffModeLine => "Set diff mode line".to_string(),
        InputAction::SetDiffModeWord => "Set diff mode word".to_string(),
        InputAction::SetDiffOnly => "Set diff line only (line/word diff only)".to_string(),

        // Output Mode
        InputAction::ToggleOutputMode => "Toggle output mode".to_string(),
        InputAction::SetOutputModeOutput => "Set output mode output".to_string(),
        InputAction::SetOutputModeStdout => "Set output mode stdout".to_string(),
        InputAction::SetOutputModeStderr => "Set output mode stderr".to_string(),

        // Keyword search
        InputAction::NextKeyword => "Forcus next keyword".to_string(),
        InputAction::PrevKeyword => "Forcus previous keyword".to_string(),

        // Toggle Wrap
        InputAction::ToggleWrapMode => "Toggle wrap mode".to_string(),

        // HistorySummary
        InputAction::ToggleHistorySummary => "Toggle history summary".to_string(),

        // Interval
        InputAction::IntervalPlus => "Interval +0.5sec".to_string(),
        InputAction::IntervalMinus => "Interval -0.5sec".to_string(),

        // Command/Filter
        InputAction::ChangeFilterMode => "Change filter mode".to_string(),
        InputAction::ChangeRegexFilterMode => "Change regex filter mode".to_string(),

        // Mouse
        InputAction::MouseScrollUp => "Mouse Scroll Up".to_string(),
        InputAction::MouseScrollDown => "Mouse Scroll Down".to_string(),
        InputAction::MouseButtonLeft => "Mouse Button Left".to_string(),
        InputAction::MouseButtonRight => "Mouse Button Right".to_string(),
        InputAction::MouseMoveLeft => "Mouse Move Left".to_string(),
        InputAction::MouseMoveRight => "Mouse Move Right".to_string(),
        InputAction::MouseMoveUp => "Mouse Move Up".to_string(),
        InputAction::MouseMoveDown => "Mouse Move Down".to_string(),
    }
}
