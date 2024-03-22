// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    style::Style,
    prelude::Line,
    widgets::{Paragraph, Wrap},
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
        let block = Paragraph::new(self.data.clone())
            .style(Style::default())
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
        // self.position = std::cmp::min(self.position + num, self.lines - 1);
        self.position = std::cmp::min(self.position + num, self.lines - self.area.height as i16);
    }

    pub fn scroll_home(&mut self) {
        self.position = 0;
    }

    pub fn scroll_end(&mut self) {
        self.position = self.lines - self.area.height as i16;
    }

}
