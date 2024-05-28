// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::{collections::HashMap, fmt::Debug};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState};
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, FileFormat};

use crate::errors::HwatchError;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl Key {
    pub fn to_str(&self) -> String {
        let modifiers = self
            .modifiers
            .iter()
            .filter_map(modifier_to_string)
            .collect::<Vec<&str>>()
            .join("-");
        let code = keycode_to_string(self.code).unwrap();
        if modifiers.is_empty() {
            code
        } else {
            format!("{}-{}", modifiers, code)
        }
    }
}

const DEFAULT_KEYMAP: [&str; 33] = [
    "up=up",  // Up
    "down=down", // Down
    "pageup=page_up", // PageUp
    "pagedown=page_down", // PageDown
    "home=move_top", // MoveTop: Home
    "end=move_end", // MoveEnd: End
    "tab=toggle_forcus", // ToggleForcus: Tab
    "left=forcus_watch_pane", // ForcusWatchPane: Left
    "right=forcus_history_pane", // ForcusHistoryPane: Right
    "q=quit", // Quit: q
    "esc=reset", // Reset: ESC
    "ctrl-c=cancel", // Cancel: Ctrl + c
    "h=help", // Help: h
    "c=toggle_color", // Toggle Color: c
    "n=toggle_line_number", // Toggle Line Number: n
    "r=toggle_reverse", // Toggle Reverse: r
    "m=toggle_mouse_support", // Toggle Mouse Support: m
    "t=toggle_view_pane_ui", // Toggle View Pane UI: t
    "backspace=toggle_view_history_pane", // Toggle View History Pane: Backspace
    "d=toggle_diff_mode", // Toggle Diff Mode: d
    "0=set_diff_mode_plane", // Set Diff Mode Plane: 0
    "1=set_diff_mode_watch", // Set Diff Mode Watch: 1
    "2=set_diff_mode_line", // Set Diff Mode Line: 2
    "3=set_diff_mode_word", // Set Diff Mode Word: 3
    "shift-o=set_diff_only", // Set Diff Only: Shift + o
    "o=toggle_output_mode", // Toggle Output Mode: o
    "f3=set_output_mode_output", // Set Output Mode Output: F3
    "f1=set_output_mode_stdout", // Set Output Mode Stdout: F1
    "f2=set_output_mode_stderr", // Set Output Mode Stderr: F2
    "plus=interval_plus", // Interval Plus: +
    "minus=interval_minus", // Interval Minus: -
    "/=change_filter_mode", // Change Filter Mode: /
    "*=change_regex_filter_mode", // Change Regex Filter Mode: *
];

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
            "tab" => KeyCode::Tab,
            c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
            _ => {
                return Err(D::Error::custom(HwatchError::ConfigError));
            }
        };
        Ok(Key { code, modifiers })
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
    pub key: Key,
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

    let config = builder
        .build()?;

    let keys = config
        .try_deserialize::<HashMap<Key, InputAction>>()?;

    for (k, a) in keys {
        // Create KeyEvent
        let key_event = KeyEvent {
            code: k.code,
            modifiers: k.modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        // Insert InputEventContents
        keymap.insert(
            Event::Key(key_event),
            InputEventContents {
                key: k,
                action: a,
            },
        );
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

    // Interval
    // ==========
    #[serde(rename = "interval_plus")]
    IntervalPlus,
    #[serde(rename = "interval_minus")]
    IntervalMinus,

    // Command
    // ==========
    #[serde(rename = "change_filter_mode")]
    ChangeFilterMode,
    #[serde(rename = "change_regex_filter_mode")]
    ChangeRegexFilterMode,

    // Input
    // ==========
}

pub fn get_input_action_description(input_action: InputAction) -> String {
    match input_action {
        // Up
        InputAction::Up => "Move up".to_string(),
        InputAction::WatchPaneUp => "Move up in watch pane".to_string(),
        InputAction::HistoryPaneUp => "Move up in history pane".to_string(),

        // Down
        InputAction::Down => "Move down".to_string(),
        InputAction::WatchPaneDown => "Move down in watch pane".to_string(),
        InputAction::HistoryPaneDown => "Move down in history pane".to_string(),

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

        // Interval
        InputAction::IntervalPlus => "Interval +0.5sec".to_string(),
        InputAction::IntervalMinus => "Interval -0.5sec".to_string(),

        // Command
        InputAction::ChangeFilterMode => "Change filter mode".to_string(),
        InputAction::ChangeRegexFilterMode => "Change regex filter mode".to_string(),

        // Input
    }

}
