// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    style::{Style, Color},
    prelude::Line,
    symbols,
    widgets::{Paragraph, Wrap, Block, Borders},
    Frame,
};

#[derive(Clone)]
pub struct WatchArea<'a> {
    ///
    area: tui::layout::Rect,

    ///
    pub data: Vec<Line<'a>>,

    ///
    position: i16,

    ///
    lines: i16,
}

/// Watch Area Object Trait
impl<'a> WatchArea<'a> {
    ///
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),

            data: vec![Line::from("")],

            position: 0,

            lines: 0,
        }
    }

    ///
    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    ///
    pub fn get_area_size(&mut self) -> i16 {
        let height = self.area.height as i16;

        return height
    }

    ///
    pub fn update_output(&mut self, data: Vec<Line<'a>>) {
        self.data = data;
    }

    ///
    pub fn draw(&mut self, frame: &mut Frame) {
        // debug
        // NOTE: 試しに枠で区切って様子見中。 最終的にはオプションでどうにかする。
        // NOTE: もし線で対処するなら↓を参考に、うまいことくっつけてきれいにしたいかも？
        //       https://ratatui.rs/how-to/layout/collapse-borders/
        let collapsed_top_and_left_border_set = symbols::border::Set {
            top_right: symbols::line::NORMAL.horizontal_down,
            ..symbols::border::PLAIN
        };
        let paragraph_block = Block::default().borders(Borders::RIGHT | Borders::TOP).border_style(Style::default().fg(Color::DarkGray)).border_set(collapsed_top_and_left_border_set);
        let block = Paragraph::new(self.data.clone())
            .style(Style::default())
            .block(paragraph_block)
            .wrap(Wrap { trim: false })
            .scroll((self.position as u16, 0));
        self.lines = block.line_count(self.area.width) as i16;
        frame.render_widget(block, self.area);
    }

    ///
    pub fn scroll_up(&mut self, num: i16) {
        self.position = std::cmp::max(0, self.position - num);
    }

    ///
    pub fn scroll_down(&mut self, num: i16) {
        if self.lines > self.area.height as i16 {
            self.position = std::cmp::min(self.position + num, self.lines - self.area.height as i16);
        }
    }

    ///
    pub fn scroll_home(&mut self) {
        self.position = 0;
    }

    ///
    pub fn scroll_end(&mut self) {
        if self.lines > self.area.height as i16 {
            self.position = self.lines - self.area.height as i16;
        }
    }

}
