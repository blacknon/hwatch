// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

#[path = "keymap_codec.rs"]
mod codec;
#[path = "keymap_defaults.rs"]
mod defaults;
#[path = "keymap_description.rs"]
mod description;

pub use self::defaults::{default_keymap, generate_keymap};
pub use self::description::get_input_action_description;

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

const DEFAULT_KEYMAP: [&str; 48] = [
    "up=up",                                    // Up
    "down=down",                                // Down
    "pageup=page_up",                           // PageUp
    "pagedown=page_down",                       // PageDown
    "home=move_top",                            // MoveTop: Home
    "end=move_end",                             // MoveEnd: End
    "tab=toggle_focus",                         // ToggleFocus: Tab
    "left=focus_watch_pane",                    // FocusWatchPane: Left
    "right=focus_history_pane",                 // FocusHistoryPane: Right
    "alt-left=scroll_left",                     // Watch window scrll left: Alt + Left
    "shift-alt-left=scroll_horizontal_home",    // Watch window scrll End left: Shift + Alt + Left
    "alt-right=scroll_right",                   // Watch window scrll right: Alt + Right
    "shift-alt-right=scroll_horizontal_end",    // Watch window scrll End right: Shift + Alt + Right
    "q=quit",                                   // Quit: q
    "esc=reset",                                // Reset: ESC
    "shift-d=delete",                           // Delete: Shift + d
    "shift-x=clear_except_selected",            // Clear Except Selected: Shift + x
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
    "w=toggle_wrap_mode",                       // Toggle Wrap Mode: w
    "f3=set_output_mode_output",                // Set Output Mode Output: F3
    "f1=set_output_mode_stdout",                // Set Output Mode Stdout: F1
    "f2=set_output_mode_stderr",                // Set Output Mode Stderr: F2
    "ctrl-n=next_keyword",                      // Next Keyword: Ctrl + n
    "ctrl-p=prev_keyword",                      // Previous Keyword: Ctrl + p
    "shift-s=toggle_history_summary",           // Toggle History Summary: Shift + s
    "plus=interval_plus",                       // Interval Plus: +
    "minus=interval_minus",                     // Interval Minus: -
    "p=toggle_pause",                           // Toggle Pause: p
    "/=change_filter_mode",                     // Change Filter Mode: /
    "*=change_regex_filter_mode",               // Change Regex Filter Mode: *
    "mouse-scroll_up=mouse_scroll_up",          // Mouse Scroll Up: Mouse Scroll Up
    "mouse-scroll_down=mouse_scroll_down",      // Mouse Scroll Down: Mouse Scroll Down
    "mouse-button_down_left=mouse_button_left", // Mouse Button Left: Mouse Button Left
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct InputEventContents {
    pub input: Input,
    pub action: InputAction,
}

pub type Keymap = HashMap<Event, InputEventContents>;

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
    #[serde(rename = "scroll_horizontal_end")]
    ScrollHorizontalEnd,

    // Scroll left
    // ==========
    #[serde(rename = "scroll_left")]
    ScrollLeft,
    #[serde(rename = "scroll_horizontal_home")]
    ScrollHorizontalHome,

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

    // Focus
    // ==========
    #[serde(rename = "toggle_focus")]
    ToggleFocus,
    #[serde(rename = "focus_watch_pane")]
    FocusWatchPane,
    #[serde(rename = "focus_history_pane")]
    FocusHistoryPane,

    // quit
    // ==========
    #[serde(rename = "quit")]
    Quit,

    // reset
    // ==========
    #[serde(rename = "reset")]
    Reset,

    // delete
    // ==========
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "clear_except_selected")]
    ClearExceptSelected,

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
    #[serde(rename = "toggle_history_summary")]
    ToggleHistorySummary,

    // Interval
    // ==========
    #[serde(rename = "interval_plus")]
    IntervalPlus,
    #[serde(rename = "interval_minus")]
    IntervalMinus,
    #[serde(rename = "toggle_pause")]
    TogglePause,

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_to_str_returns_fallback_for_unsupported_keycode() {
        let input = Input {
            input: InputType::Key(Key {
                code: KeyCode::F(13),
                modifiers: KeyModifiers::CONTROL,
            }),
        };

        assert_eq!(input.to_str(), "ctrl-unknown");
    }

    #[test]
    fn key_deserialize_rejects_empty_key_name_without_panicking() {
        let err = Key::deserialize(serde::de::value::StringDeserializer::<
            serde::de::value::Error,
        >::new("".to_string()));

        assert!(err.is_err());
    }

    #[test]
    fn default_keymap_contains_expected_bindings() {
        let keymap = default_keymap();

        assert!(!keymap.is_empty());
        assert!(keymap.values().any(|v| v.action == InputAction::Quit));
        assert!(keymap
            .values()
            .any(|v| v.action == InputAction::TogglePause));
    }
}
