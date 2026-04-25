// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::InputAction;

pub fn get_input_action_description(input_action: InputAction) -> String {
    match input_action {
        InputAction::None => "No action".to_string(),
        InputAction::Up => "Move up".to_string(),
        InputAction::WatchPaneUp => "Move up in watch pane".to_string(),
        InputAction::HistoryPaneUp => "Move up in history pane".to_string(),
        InputAction::Down => "Move down".to_string(),
        InputAction::WatchPaneDown => "Move down in watch pane".to_string(),
        InputAction::HistoryPaneDown => "Move down in history pane".to_string(),
        InputAction::ScrollRight => "Move Right".to_string(),
        InputAction::ScrollHorizontalEnd => "Move Right end".to_string(),
        InputAction::ScrollLeft => "Move Left".to_string(),
        InputAction::ScrollHorizontalHome => "Move Left home".to_string(),
        InputAction::PageUp => "Move page up".to_string(),
        InputAction::WatchPanePageUp => "Move page up in watch pane".to_string(),
        InputAction::HistoryPanePageUp => "Move page up in history pane".to_string(),
        InputAction::PageDown => "Move page down".to_string(),
        InputAction::WatchPanePageDown => "Move page down in watch pane".to_string(),
        InputAction::HistoryPanePageDown => "Move page down in history pane".to_string(),
        InputAction::MoveTop => "Move top".to_string(),
        InputAction::WatchPaneMoveTop => "Move top in watch pane".to_string(),
        InputAction::HistoryPaneMoveTop => "Move top in history pane".to_string(),
        InputAction::MoveEnd => "Move end".to_string(),
        InputAction::WatchPaneMoveEnd => "Move end in watch pane".to_string(),
        InputAction::HistoryPaneMoveEnd => "Move end in history pane".to_string(),
        InputAction::ToggleFocus => "Toggle focus window".to_string(),
        InputAction::FocusWatchPane => "Focus watch pane".to_string(),
        InputAction::FocusHistoryPane => "Focus history pane".to_string(),
        InputAction::Quit => "Quit hwatch".to_string(),
        InputAction::Reset => "filter reset".to_string(),
        InputAction::Delete => "Delete selected history".to_string(),
        InputAction::ClearExceptSelected => {
            "Clear all history except selected history".to_string()
        }
        InputAction::Cancel => "Cancel".to_string(),
        InputAction::ForceCancel => "Cancel without displaying the exit dialog".to_string(),
        InputAction::Help => "Show and hide help window".to_string(),
        InputAction::ToggleColor => "Toggle enable/disable ANSI Color".to_string(),
        InputAction::ToggleLineNumber => "Toggle enable/disable Line Number".to_string(),
        InputAction::ToggleReverse => "Toggle enable/disable text reverse".to_string(),
        InputAction::ToggleMouseSupport => "Toggle enable/disable mouse support".to_string(),
        InputAction::ToggleViewPaneUI => "Toggle view header/history pane".to_string(),
        InputAction::ToggleViewHeaderPane => "Toggle view header pane".to_string(),
        InputAction::ToggleViewHistoryPane => "Toggle view history pane".to_string(),
        InputAction::ToggleBorder => "Toggle enable/disable border".to_string(),
        InputAction::ToggleScrollBar => "Toggle enable/disable scroll bar".to_string(),
        InputAction::ToggleBorderWithScrollBar => {
            "Toggle enable/disable border and scroll bar".to_string()
        }
        InputAction::ToggleDiffMode => "Toggle diff mode".to_string(),
        InputAction::SetDiffModePlane => "Set diff mode plane".to_string(),
        InputAction::SetDiffModeWatch => "Set diff mode watch".to_string(),
        InputAction::SetDiffModeLine => "Set diff mode line".to_string(),
        InputAction::SetDiffModeWord => "Set diff mode word".to_string(),
        InputAction::SetDiffOnly => "Set diff line only (line/word diff only)".to_string(),
        InputAction::ToggleOutputMode => "Toggle output mode".to_string(),
        InputAction::SetOutputModeOutput => "Set output mode output".to_string(),
        InputAction::SetOutputModeStdout => "Set output mode stdout".to_string(),
        InputAction::SetOutputModeStderr => "Set output mode stderr".to_string(),
        InputAction::NextKeyword => "Focus next keyword".to_string(),
        InputAction::PrevKeyword => "Focus previous keyword".to_string(),
        InputAction::ToggleWrapMode => "Toggle wrap mode".to_string(),
        InputAction::ToggleHistorySummary => "Toggle history summary".to_string(),
        InputAction::IntervalPlus => "Interval +0.5sec".to_string(),
        InputAction::IntervalMinus => "Interval -0.5sec".to_string(),
        InputAction::TogglePause => "Toggle Execution Pause".to_string(),
        InputAction::ChangeFilterMode => "Change filter mode".to_string(),
        InputAction::ChangeRegexFilterMode => "Change regex filter mode".to_string(),
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
