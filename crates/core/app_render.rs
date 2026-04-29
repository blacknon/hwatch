// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{ActiveArea, ActiveWindow, App, InputMode};
use crate::popup::PopupWindow;
use crate::HISTORY_WIDTH;
use tui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    Frame,
};
use unicode_width::UnicodeWidthStr;

impl App<'_> {
    pub(super) fn draw(&mut self, f: &mut Frame) {
        self.define_subareas(f.area());

        if self.show_header {
            self.header_area.draw(f);
        }

        self.watch_area.draw(f);

        self.history_area
            .set_active(self.area == ActiveArea::History);
        self.history_area.draw(f);

        self.draw_overlay(f);

        if self.window == ActiveWindow::Normal {
            self.draw_filter_cursor(f);
        }
    }

    fn draw_overlay(&mut self, f: &mut Frame) {
        match self.window {
            ActiveWindow::Normal => {}
            ActiveWindow::Help => self.help_window.draw(f),
            ActiveWindow::Exit => {
                self.draw_popup(
                    f,
                    "exit",
                    vec![
                        " Exit hwatch?".to_string(),
                        "   Press 'Y' or 'Q'  : Quit.".to_string(),
                        "   Press 'N' or 'Esc': Stay.".to_string(),
                    ],
                );
            }
            ActiveWindow::Delete => {
                self.draw_popup(
                    f,
                    "delete",
                    vec![
                        " Delete this history?".to_string(),
                        "   Press 'Y'         : Delete.".to_string(),
                        "   Press 'N' or 'Esc': Stay.".to_string(),
                    ],
                );
            }
            ActiveWindow::Clear => {
                self.draw_popup(
                    f,
                    "clear",
                    vec![
                        " Clear all history except selected?".to_string(),
                        "   Press 'Y'         : Clear.".to_string(),
                        "   Press 'N' or 'Esc': Stay.".to_string(),
                    ],
                );
            }
        }
    }

    fn draw_popup(&mut self, f: &mut Frame, title: &str, lines: Vec<String>) {
        let mut popup_window = PopupWindow::new(title, lines);
        popup_window.draw(f);
    }

    fn draw_filter_cursor(&mut self, f: &mut Frame) {
        match self.input_mode {
            InputMode::Filter | InputMode::RegexFilter => {
                if self.show_header {
                    let cursor_x =
                        self.header_area.area.x + self.header_area.input_text.width() as u16 + 1;
                    let cursor_y = self.header_area.area.y + 1;

                    let area = f.area();
                    let max_x = area.width.saturating_sub(1);
                    let max_y = area.height.saturating_sub(1);
                    let x = cursor_x.min(max_x);
                    let y = cursor_y.min(max_y);

                    f.set_cursor_position(Position { x, y });
                }
            }
            _ => {}
        }
    }

    fn define_subareas(&mut self, total_area: Rect) {
        let history_width: u16 = match self.show_history {
            true => HISTORY_WIDTH,
            false => match self.area == ActiveArea::History
                || self.history_area.get_state_select() != 0
            {
                true => 2,
                false => 0,
            },
        };

        let header_height: u16 = match self.show_header {
            true => 2,
            false => 0,
        };

        let top_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(header_height),
                    Constraint::Max(total_area.height.saturating_sub(header_height)),
                ]
                .as_ref(),
            )
            .split(total_area);
        self.header_area.set_area(top_chunks[0]);

        let main_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Max(total_area.width.saturating_sub(history_width)),
                    Constraint::Length(history_width),
                ]
                .as_ref(),
            )
            .direction(Direction::Horizontal)
            .split(top_chunks[1]);

        self.watch_area.set_area(main_chunks[0]);
        self.history_area.set_area(main_chunks[1]);
    }
}
