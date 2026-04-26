// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{ActiveArea, ActiveWindow, App, InputMode};
use crate::common::OutputMode;
use crate::event::AppEvent;
use regex::Regex;
use tui::layout::Rect;

impl App<'_> {
    pub(super) fn set_area(&mut self, target: ActiveArea) {
        self.area = target;
        self.header_area.set_active_area(self.area);
        self.header_area.update();
    }

    pub(super) fn toggle_area(&mut self) {
        if let ActiveWindow::Normal = self.window {
            match self.area {
                ActiveArea::Watch => self.set_area(ActiveArea::History),
                ActiveArea::History => self.set_area(ActiveArea::Watch),
            }
        }
    }

    pub(super) fn toggle_output(&mut self) {
        match self.output_mode {
            OutputMode::Output => self.set_output_mode(OutputMode::Stdout),
            OutputMode::Stdout => self.set_output_mode(OutputMode::Stderr),
            OutputMode::Stderr => self.set_output_mode(OutputMode::Output),
        }
    }

    pub(super) fn toggle_diff_mode(&mut self) {
        self.diff_mode = if self.diff_mode + 1 > self.diff_modes.len() - 1 {
            0
        } else {
            self.diff_mode + 1
        };
        self.set_diff_mode(self.diff_mode);
    }

    pub(super) fn toggle_window(&mut self) {
        match self.window {
            ActiveWindow::Normal => self.window = ActiveWindow::Help,
            ActiveWindow::Help => self.window = ActiveWindow::Normal,
            _ => {}
        }
    }

    pub(super) fn show_exit_popup(&mut self) {
        self.window = ActiveWindow::Exit;
    }

    pub(super) fn show_delete_popup(&mut self) {
        self.window = ActiveWindow::Delete;
    }

    pub(super) fn show_clear_popup(&mut self) {
        self.window = ActiveWindow::Clear;
    }

    pub(super) fn matches_filter_text(&self, result_text: &str) -> bool {
        if self.is_regex_filter {
            Regex::new(&self.filtered_text)
                .map(|re| re.is_match(result_text))
                .unwrap_or(false)
        } else {
            result_text.contains(&self.filtered_text)
        }
    }

    pub(crate) fn show_history(&mut self, visible: bool) {
        self.show_history = visible;
        if !visible {
            self.set_area(ActiveArea::Watch);
        }
        let _ = self.tx.send(AppEvent::Redraw);
    }

    pub(super) fn add_history(&mut self, result_index: usize, selected: usize) {
        let results = match self.output_mode {
            OutputMode::Output => &self.results,
            OutputMode::Stdout => &self.results_stdout,
            OutputMode::Stderr => &self.results_stderr,
        };

        let timestamp = &results[&result_index].command_result.timestamp;
        let status = &results[&result_index].command_result.status;

        let history_summary = results[&result_index].summary.clone();

        self.history_area.update(
            timestamp.to_string(),
            *status,
            result_index as u16,
            history_summary,
        );

        if selected != 0 {
            self.history_area.previous(1);
        }
    }

    pub(crate) fn show_ui(&mut self, visible: bool) {
        self.show_header = visible;
        self.show_history = visible;

        self.history_area.set_hide_header(!visible);
        self.watch_area.set_hide_header(!visible);

        let _ = self.tx.send(AppEvent::Redraw);
    }

    pub(crate) fn show_help_banner(&mut self, visible: bool) {
        self.header_area.set_banner(
            if visible {
                "Display help with h key!"
            } else {
                ""
            }
            .to_string(),
        );
        let _ = self.tx.send(AppEvent::Redraw);
    }

    pub(super) fn action_normal_reset(&mut self) {
        if self.is_filtered {
            self.is_filtered = false;
            self.is_regex_filter = false;
            self.filtered_text = "".to_string();
            self.header_area.input_text = self.filtered_text.clone();
            self.set_input_mode(InputMode::None);

            let selected = self.history_area.get_state_select();
            let new_selected = self.reset_history(selected);

            self.watch_area.reset_keyword();
            self.set_output_data(new_selected);
        } else if 0 != self.history_area.get_state_select() {
            self.reset_history(0);
            self.set_output_data(0);
        } else {
            self.show_exit_popup()
        }
    }

    pub(super) fn action_force_reset(&mut self) {
        if self.is_filtered {
            self.is_filtered = false;
            self.is_regex_filter = false;
            self.filtered_text = "".to_string();
            self.header_area.input_text = self.filtered_text.clone();
            self.set_input_mode(InputMode::None);

            let selected = self.history_area.get_state_select();
            let new_selected = self.reset_history(selected);

            self.watch_area.reset_keyword();
            self.set_output_data(new_selected);
        } else if 0 != self.history_area.get_state_select() {
            self.reset_history(0);
            self.set_output_data(0);
        } else {
            self.exit();
        }
    }

    pub(super) fn action_up(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => self.action_watch_up(),
                ActiveArea::History => self.action_history_up(),
            },
            ActiveWindow::Help => {
                self.help_window.scroll_up(1);
            }
            _ => {}
        }
    }

    pub(super) fn action_watch_up(&mut self) {
        self.watch_area.scroll_up(1);
    }

    pub(super) fn action_history_up(&mut self) {
        self.history_area.next(1);
        self.refresh_selected_watch_output();
    }

    pub(super) fn action_down(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => self.action_watch_down(),
                ActiveArea::History => self.action_history_down(),
            },
            ActiveWindow::Help => {
                self.help_window.scroll_down(1);
            }
            _ => {}
        }
    }

    pub(super) fn action_watch_down(&mut self) {
        self.watch_area.scroll_down(1);
    }

    pub(super) fn action_history_down(&mut self) {
        self.history_area.previous(1);
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub(super) fn action_pgup(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {
                    self.action_watch_pgup();
                }
                ActiveArea::History => {
                    self.action_history_pgup();
                }
            },
            ActiveWindow::Help => {
                self.help_window.page_up();
            }
            _ => {}
        }
    }

    pub(super) fn action_watch_pgup(&mut self) {
        let mut page_height = self.watch_area.get_area_size();
        if page_height > 1 {
            page_height -= 1
        }
        self.watch_area.scroll_up(page_height);
    }

    pub(super) fn action_history_pgup(&mut self) {
        let area_size = self.history_area.area.height;
        let move_size = if area_size > 1 { area_size - 1 } else { 1 };
        self.history_area.next(move_size as usize);
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub(super) fn action_pgdn(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {
                    self.action_watch_pgdn();
                }
                ActiveArea::History => {
                    self.action_history_pgdn();
                }
            },
            ActiveWindow::Help => {
                self.help_window.page_down();
            }
            _ => {}
        }
    }

    pub(super) fn action_watch_pgdn(&mut self) {
        let mut page_height = self.watch_area.get_area_size();
        if page_height > 1 {
            page_height -= 1
        }
        self.watch_area.scroll_down(page_height);
    }

    pub(super) fn action_history_pgdn(&mut self) {
        let area_size = self.history_area.area.height;
        let move_size = if area_size > 1 { area_size - 1 } else { 1 };
        self.history_area.previous(move_size as usize);
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub(super) fn action_top(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => self.watch_area.scroll_home(),
                ActiveArea::History => self.action_history_top(),
            },
            ActiveWindow::Help => {
                self.help_window.scroll_top();
            }
            _ => {}
        }
    }

    pub(super) fn action_history_top(&mut self) {
        let hisotory_size = self.history_area.get_history_size();
        self.history_area.next(hisotory_size);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub(super) fn action_end(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => self.watch_area.scroll_end(),
                ActiveArea::History => self.action_history_end(),
            },
            ActiveWindow::Help => {
                self.help_window.scroll_end();
            }
            _ => {}
        }
    }

    pub(super) fn action_history_end(&mut self) {
        let hisotory_size = self.history_area.get_history_size();
        let move_size = if hisotory_size > 1 {
            hisotory_size - 1
        } else {
            1
        };

        self.history_area.previous(move_size);
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub(super) fn action_input_reset(&mut self) {
        self.header_area.input_text = self.filtered_text.clone();
        self.set_input_mode(InputMode::None);

        let selected = self.history_area.get_state_select();
        let new_selected = self.reset_history(selected);
        self.set_output_data(new_selected);
    }

    pub(super) fn action_previous_keyword(&mut self) {
        self.watch_area.previous_keyword();
    }

    pub(super) fn action_delete_history(&mut self) {
        let selected = self.history_area.get_state_select();
        if selected != 0 && self.history_area.get_history_size() > 0 {
            self.delete_output_data(selected);

            let new_selected = self.reset_history(selected);
            self.set_output_data(new_selected);
        }
    }

    pub(super) fn action_next_keyword(&mut self) {
        self.watch_area.next_keyword();
    }

    pub(super) fn select_watch_pane(&mut self) {
        if let ActiveWindow::Normal = self.window {
            self.area = ActiveArea::Watch;
            self.header_area.set_active_area(self.area);
            self.header_area.update();
        }
    }

    pub(super) fn select_history_pane(&mut self) {
        if let ActiveWindow::Normal = self.window {
            self.area = ActiveArea::History;
            self.header_area.set_active_area(self.area);
            self.header_area.update();
        }
    }

    pub(super) fn mouse_click_left(&mut self, column: u16, row: u16) {
        let is_history_area = check_in_area(self.history_area.area, column, row);
        if is_history_area {
            let headline_count = self.history_area.area.y;
            self.history_area.click_row(row - headline_count);

            let selected = self.history_area.get_state_select();
            self.set_output_data(selected);
        }
    }

    pub(super) fn mouse_scroll_up(&mut self, column: u16, row: u16) {
        match self.window {
            ActiveWindow::Normal => {
                if column == 0 && row == 0 {
                    match self.area {
                        ActiveArea::Watch => {
                            self.watch_area.scroll_up(2);
                        }
                        ActiveArea::History => {
                            self.history_area.next(1);

                            let selected = self.history_area.get_state_select();
                            self.set_output_data(selected);
                        }
                    }
                } else {
                    let is_history_area = check_in_area(self.history_area.area, column, row);
                    if is_history_area {
                        self.history_area.next(1);

                        let selected = self.history_area.get_state_select();
                        self.set_output_data(selected);
                    } else {
                        self.watch_area.scroll_up(2);
                    }
                }
            }
            ActiveWindow::Help => {
                self.help_window.scroll_up(2);
            }
            _ => {}
        }
    }

    pub(super) fn mouse_scroll_down(&mut self, column: u16, row: u16) {
        match self.window {
            ActiveWindow::Normal => {
                if column == 0 && row == 0 {
                    match self.area {
                        ActiveArea::Watch => {
                            self.watch_area.scroll_down(2);
                        }
                        ActiveArea::History => {
                            self.history_area.previous(1);

                            let selected = self.history_area.get_state_select();
                            self.set_output_data(selected);
                        }
                    }
                } else {
                    let is_history_area = check_in_area(self.history_area.area, column, row);
                    if is_history_area {
                        self.history_area.previous(1);

                        let selected = self.history_area.get_state_select();
                        self.set_output_data(selected);
                    } else {
                        self.watch_area.scroll_down(2);
                    }
                }
            }
            ActiveWindow::Help => {
                self.help_window.scroll_down(2);
            }
            _ => {}
        }
    }

    pub(super) fn exit(&mut self) {
        self.tx
            .send(AppEvent::Exit)
            .expect("send error hwatch exit.");
    }
}

fn check_in_area(area: Rect, column: u16, row: u16) -> bool {
    let mut result = true;

    let area_top = area.top();
    let area_bottom = area.bottom();
    let area_left = area.left();
    let area_right = area.right();

    let area_row_range = area_top..area_bottom;
    let area_column_range = area_left..area_right;

    if !area_row_range.contains(&row) || !area_column_range.contains(&column) {
        result = false;
    }

    result
}
