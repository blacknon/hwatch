// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::{collections::HashMap, fmt::Debug};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

use crate::errors::HwatchError;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
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

pub type Keymap = HashMap<Key, InputAction>;

pub fn default_keymap() -> Keymap {
    HashMap::from([
        // Up
        ( Key { code: KeyCode::Up, modifiers: KeyModifiers::NONE }, InputAction::Up ),

        // Down
        ( Key { code: KeyCode::Down, modifiers: KeyModifiers::NONE }, InputAction::Down ),

        // PageUp
        ( Key { code: KeyCode::PageUp, modifiers: KeyModifiers::NONE }, InputAction::PageUp ),

        // PageDown
        ( Key { code: KeyCode::PageDown, modifiers: KeyModifiers::NONE }, InputAction::PageDown ),

        // Move Top: Home
        ( Key { code: KeyCode::Home, modifiers: KeyModifiers::NONE }, InputAction::MoveTop ),

        // Move End: End
        ( Key { code: KeyCode::End, modifiers: KeyModifiers::NONE }, InputAction::MoveEnd ),

        // ToggleForcus: Tab
        ( Key { code: KeyCode::Tab, modifiers: KeyModifiers::NONE }, InputAction::ToggleForcus ),

        // Forcus Watch Pane: Left
        ( Key { code: KeyCode::Left, modifiers: KeyModifiers::NONE }, InputAction::ForcusWatchPane ),

        // Forcus History Pane: Right
        ( Key { code: KeyCode::Right, modifiers: KeyModifiers::NONE }, InputAction::ForcusHistoryPane ),

        // Quit: q
        ( Key { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE }, InputAction::Quit ),

        // Reset: ESC
        ( Key { code: KeyCode::Esc, modifiers: KeyModifiers::NONE }, InputAction::Reset ),

        // Cancel: Ctrl + c
        ( Key { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL }, InputAction::Cancel ),

        // Help: h
        ( Key { code: KeyCode::Char('h'), modifiers: KeyModifiers::NONE }, InputAction::Help ),

        // Toggle Color: c
        ( Key { code: KeyCode::Char('c'), modifiers: KeyModifiers::NONE }, InputAction::ToggleColor ),

        // Toggle Line Number: n
        ( Key { code: KeyCode::Char('n'), modifiers: KeyModifiers::NONE }, InputAction::ToggleLineNumber ),

        // Toggle Reverse: r
        ( Key { code: KeyCode::Char('r'), modifiers: KeyModifiers::NONE }, InputAction::ToggleReverse ),

        // Toggle Mouse Support: m
        ( Key { code: KeyCode::Char('m'), modifiers: KeyModifiers::NONE }, InputAction::ToggleMouseSupport ),

        // Toggle View Pane UI: t
        ( Key { code: KeyCode::Char('t'), modifiers: KeyModifiers::NONE }, InputAction::ToggleViewPaneUI ),

        // Toggle View History Pane: Backspace
        ( Key { code: KeyCode::Backspace, modifiers: KeyModifiers::NONE }, InputAction::ToggleViewHistoryPane ),

        // Diff Mode
        // ==========
        // Toggle Diff Mode: d
        ( Key { code: KeyCode::Char('d'), modifiers: KeyModifiers::NONE }, InputAction::ToggleDiffMode ),

        // Set Diff Mode Plane: 0
        ( Key { code: KeyCode::Char('0'), modifiers: KeyModifiers::NONE }, InputAction::SetDiffModePlane ),

        // Set Diff Mode Watch: 1
        ( Key { code: KeyCode::Char('1'), modifiers: KeyModifiers::NONE }, InputAction::SetDiffModeWatch ),

        // Set Diff Mode Line: 2
        ( Key { code: KeyCode::Char('2'), modifiers: KeyModifiers::NONE }, InputAction::SetDiffModeLine ),

        // Set Diff Mode Word: 3
        ( Key { code: KeyCode::Char('3'), modifiers: KeyModifiers::NONE }, InputAction::SetDiffModeWord ),

        // Set Diff Only: Shift + o
        ( Key { code: KeyCode::Char('o'), modifiers: KeyModifiers::SHIFT }, InputAction::SetDiffOnly ),

        // Output Mode
        // ==========
        // Toggle Output Mode: o
        ( Key { code: KeyCode::Char('o'), modifiers: KeyModifiers::NONE }, InputAction::ToggleOutputMode ),

        // Set Output Mode Output: F3
        ( Key { code: KeyCode::F(3), modifiers: KeyModifiers ::NONE }, InputAction::SetOutputModeOutput ),

        // Set Output Mode Stdout: F2
        ( Key { code: KeyCode::F(1), modifiers: KeyModifiers ::NONE }, InputAction::SetOutputModeStdout ),

        // Set Output Mode Stderr: F3
        ( Key { code: KeyCode::F(2), modifiers: KeyModifiers ::NONE }, InputAction::SetOutputModeStderr ),

        // Interval
        // ==========
        // Interval Plus: +
        ( Key { code: KeyCode::Char('+'), modifiers: KeyModifiers::NONE }, InputAction::IntervalPlus ),

        // Interval Minus: -
        ( Key { code: KeyCode::Char('-'), modifiers: KeyModifiers::NONE }, InputAction::IntervalMinus ),

        // Command
        // ==========
        // Change Filter Mode: /
        ( Key { code: KeyCode::Char('/'), modifiers: KeyModifiers::NONE }, InputAction::ChangeFilterMode ),

        // Change Regex Filter Mode: *
        ( Key { code: KeyCode::Char('*'), modifiers: KeyModifiers::CONTROL }, InputAction::ChangeRegexFilterMode ),
    ])
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
    #[serde(rename = "toggle_view_hisotry_pane")]
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
}
