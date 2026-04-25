// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{ActiveWindow, App, InputMode};
use crate::common::OutputMode;
use crate::keymap::{InputAction, InputEventContents};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEvent};
use regex::Regex;

impl App<'_> {
    pub(super) fn get_event(&mut self, terminal_event: crossterm::event::Event) {
        match self.input_mode {
            InputMode::None => self.get_normal_input_key(terminal_event),
            InputMode::Filter => self.get_filter_input_key(false, terminal_event),
            InputMode::RegexFilter => self.get_filter_input_key(true, terminal_event),
        }
    }

    pub(super) fn get_input_action(
        &self,
        terminal_event: &crossterm::event::Event,
    ) -> Option<&InputEventContents> {
        match *terminal_event {
            Event::Key(_) => self.keymap.get(terminal_event),
            Event::Mouse(mouse) => {
                let mouse_event = MouseEvent {
                    kind: mouse.kind,
                    column: 0,
                    row: 0,
                    modifiers: KeyModifiers::empty(),
                };
                self.keymap.get(&Event::Mouse(mouse_event))
            }
            _ => None,
        }
    }

    pub(super) fn get_normal_input_key(&mut self, terminal_event: crossterm::event::Event) {
        if self.window == ActiveWindow::Exit {
            if let Event::Key(key) = terminal_event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('q') => {
                            self.exit();
                            return;
                        }
                        KeyCode::Char('n') => {
                            self.window = ActiveWindow::Normal;
                            return;
                        }
                        KeyCode::Char('h') => {
                            self.window = ActiveWindow::Help;
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.window == ActiveWindow::Delete {
            if let Event::Key(key) = terminal_event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('y') => {
                            self.action_delete_history();
                            self.window = ActiveWindow::Normal;
                            return;
                        }
                        KeyCode::Char('n') => {
                            self.window = ActiveWindow::Normal;
                            return;
                        }
                        KeyCode::Char('h') => {
                            self.window = ActiveWindow::Help;
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.window == ActiveWindow::Clear {
            if let Event::Key(key) = terminal_event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('y') => {
                            self.clear_history_except_selected();
                            self.window = ActiveWindow::Normal;
                            return;
                        }
                        KeyCode::Char('n') => {
                            self.window = ActiveWindow::Normal;
                            return;
                        }
                        KeyCode::Char('h') => {
                            self.window = ActiveWindow::Help;
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(event_content) = self.get_input_action(&terminal_event) {
            let action = event_content.action;
            match self.window {
                ActiveWindow::Normal => match action {
                    InputAction::Up => self.action_up(),
                    InputAction::WatchPaneUp => self.action_watch_up(),
                    InputAction::HistoryPaneUp => self.action_history_up(),
                    InputAction::Down => self.action_down(),
                    InputAction::WatchPaneDown => self.action_watch_down(),
                    InputAction::HistoryPaneDown => self.action_history_down(),
                    InputAction::ScrollRight => self.watch_area.scroll_right(1),
                    InputAction::ScrollHorizontalEnd => self.watch_area.scroll_horizontal_end(),
                    InputAction::ScrollLeft => self.watch_area.scroll_left(1),
                    InputAction::ScrollHorizontalHome => self.watch_area.scroll_horizontal_home(),
                    InputAction::PageUp => self.action_pgup(),
                    InputAction::WatchPanePageUp => self.action_watch_pgup(),
                    InputAction::HistoryPanePageUp => self.action_history_pgup(),
                    InputAction::PageDown => self.action_pgdn(),
                    InputAction::WatchPanePageDown => self.action_watch_pgdn(),
                    InputAction::HistoryPanePageDown => self.action_history_pgdn(),
                    InputAction::MoveTop => self.action_top(),
                    InputAction::WatchPaneMoveTop => self.watch_area.scroll_home(),
                    InputAction::HistoryPaneMoveTop => self.action_history_top(),
                    InputAction::MoveEnd => self.action_end(),
                    InputAction::WatchPaneMoveEnd => self.watch_area.scroll_end(),
                    InputAction::HistoryPaneMoveEnd => self.action_history_end(),
                    InputAction::ToggleFocus => self.toggle_area(),
                    InputAction::FocusWatchPane => self.select_watch_pane(),
                    InputAction::FocusHistoryPane => self.select_history_pane(),
                    InputAction::Quit => {
                        if self.disable_exit_dialog {
                            self.exit();
                        } else {
                            self.show_exit_popup();
                        }
                    }
                    InputAction::Reset => self.action_normal_reset(),
                    InputAction::Delete => self.show_delete_popup(),
                    InputAction::ClearExceptSelected => self.show_clear_popup(),
                    InputAction::Cancel => self.action_normal_reset(),
                    InputAction::ForceCancel => self.action_force_reset(),
                    InputAction::Help => self.toggle_window(),
                    InputAction::ToggleColor => self.set_ansi_color(!self.ansi_color),
                    InputAction::ToggleLineNumber => self.set_line_number(!self.line_number),
                    InputAction::ToggleReverse => self.set_reverse(!self.reverse),
                    InputAction::ToggleMouseSupport => {
                        self.set_mouse_events(!self.mouse_events)
                    }
                    InputAction::ToggleViewPaneUI => self.show_ui(!self.show_header),
                    InputAction::ToggleViewHistoryPane => self.show_history(!self.show_history),
                    InputAction::ToggleBorder => self.set_border(!self.is_border),
                    InputAction::ToggleScrollBar => self.set_scroll_bar(!self.is_scroll_bar),
                    InputAction::ToggleBorderWithScrollBar => {
                        self.set_border(!self.is_border);
                        self.set_scroll_bar(!self.is_scroll_bar);
                    }
                    InputAction::ToggleDiffMode => self.toggle_diff_mode(),
                    InputAction::SetDiffModePlane => self.set_diff_mode(0),
                    InputAction::SetDiffModeWatch => self.set_diff_mode(1),
                    InputAction::SetDiffModeLine => self.set_diff_mode(2),
                    InputAction::SetDiffModeWord => self.set_diff_mode(3),
                    InputAction::SetDiffOnly => {
                        self.set_is_only_diffline(!self.is_only_diffline)
                    }
                    InputAction::ToggleOutputMode => self.toggle_output(),
                    InputAction::SetOutputModeOutput => self.set_output_mode(OutputMode::Output),
                    InputAction::SetOutputModeStdout => self.set_output_mode(OutputMode::Stdout),
                    InputAction::SetOutputModeStderr => self.set_output_mode(OutputMode::Stderr),
                    InputAction::ToggleWrapMode => self.watch_area.toggle_wrap_mode(),
                    InputAction::NextKeyword => self.action_next_keyword(),
                    InputAction::PrevKeyword => self.action_previous_keyword(),
                    InputAction::ToggleHistorySummary => {
                        self.set_history_summary(!self.is_history_summary)
                    }
                    InputAction::IntervalPlus => self.increase_interval(),
                    InputAction::IntervalMinus => self.decrease_interval(),
                    InputAction::TogglePause => self.toggle_pause(),
                    InputAction::ChangeFilterMode => self.set_input_mode(InputMode::Filter),
                    InputAction::ChangeRegexFilterMode => {
                        self.set_input_mode(InputMode::RegexFilter)
                    }
                    InputAction::MouseScrollDown => {
                        if let Event::Mouse(mouse) = terminal_event {
                            self.mouse_scroll_down(mouse.column, mouse.row)
                        } else {
                            self.mouse_scroll_down(0, 0)
                        }
                    }
                    InputAction::MouseScrollUp => {
                        if let Event::Mouse(mouse) = terminal_event {
                            self.mouse_scroll_up(mouse.column, mouse.row)
                        } else {
                            self.mouse_scroll_up(0, 0)
                        }
                    }
                    InputAction::MouseButtonLeft => {
                        if let Event::Mouse(mouse) = terminal_event {
                            self.mouse_click_left(mouse.column, mouse.row)
                        } else {
                            self.mouse_click_left(0, 0)
                        }
                    }
                    _ => {}
                },
                ActiveWindow::Help => match action {
                    InputAction::Up => self.action_up(),
                    InputAction::Down => self.action_down(),
                    InputAction::PageUp => self.action_pgup(),
                    InputAction::PageDown => self.action_pgdn(),
                    InputAction::MoveTop => self.action_top(),
                    InputAction::MoveEnd => self.action_end(),
                    InputAction::Help => self.toggle_window(),
                    InputAction::Quit => {
                        if self.disable_exit_dialog {
                            self.exit();
                        } else {
                            self.show_exit_popup();
                        }
                    }
                    InputAction::Cancel => self.toggle_window(),
                    InputAction::MouseScrollDown => {
                        if let Event::Mouse(mouse) = terminal_event {
                            self.mouse_scroll_down(mouse.column, mouse.row)
                        } else {
                            self.mouse_scroll_down(0, 0)
                        }
                    }
                    InputAction::MouseScrollUp => {
                        if let Event::Mouse(mouse) = terminal_event {
                            self.mouse_scroll_up(mouse.column, mouse.row)
                        } else {
                            self.mouse_scroll_up(0, 0)
                        }
                    }
                    _ => {}
                },
                ActiveWindow::Exit => match action {
                    InputAction::Quit => self.exit(),
                    InputAction::Cancel => self.exit(),
                    InputAction::Reset => self.window = ActiveWindow::Normal,
                    _ => {}
                },
                ActiveWindow::Delete => match action {
                    InputAction::Quit => self.action_delete_history(),
                    InputAction::Cancel => self.window = ActiveWindow::Normal,
                    InputAction::Reset => self.window = ActiveWindow::Normal,
                    _ => {}
                },
                ActiveWindow::Clear => match action {
                    InputAction::Quit => self.clear_history_except_selected(),
                    InputAction::Cancel => self.window = ActiveWindow::Normal,
                    InputAction::Reset => self.window = ActiveWindow::Normal,
                    _ => {}
                },
            }
        }
    }

    pub(super) fn get_filter_input_key(
        &mut self,
        is_regex: bool,
        terminal_event: crossterm::event::Event,
    ) {
        if let Some(event_content) = self.keymap.get(&terminal_event) {
            let action = event_content.action;
            match action {
                InputAction::Cancel => self.action_input_reset(),
                _ => self.get_default_filter_input_key(is_regex, terminal_event),
            }
        } else {
            self.get_default_filter_input_key(is_regex, terminal_event)
        }
    }

    pub(super) fn get_default_filter_input_key(
        &mut self,
        is_regex: bool,
        terminal_event: crossterm::event::Event,
    ) {
        if let Event::Key(key) = terminal_event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(c) => {
                        self.header_area.input_text.push(c);
                        self.header_area.update();
                    }
                    KeyCode::Backspace => {
                        self.header_area.input_text.pop();
                        self.header_area.update();
                    }
                    KeyCode::Enter => {
                        if is_regex {
                            let input_text = self.header_area.input_text.clone();
                            let re_result = Regex::new(&input_text);
                            if re_result.is_err() {
                                return;
                            }
                        }

                        self.is_filtered = true;
                        self.is_regex_filter = is_regex;
                        self.filtered_text = self.header_area.input_text.clone();
                        self.set_input_mode(InputMode::None);

                        let selected: usize = self.history_area.get_state_select();
                        let new_selected = self.reset_history(selected);

                        self.watch_area
                            .set_keyword(self.filtered_text.clone(), is_regex);
                        self.set_output_data(new_selected);
                    }
                    _ => {}
                }
            }
        }
    }
}
