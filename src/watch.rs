// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    backend::Backend,
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
        }
    }

    ///
    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    ///
    pub fn update_output(&mut self, data: Vec<Line<'a>>) {
        self.data = data;
    }

    ///
    pub fn draw<B: Backend>(&mut self, frame: &mut Frame) {
        let block = Paragraph::new(self.data.clone())
            .style(Style::default())
            .wrap(Wrap { trim: false })
            .scroll((self.position as u16, 0));
        frame.render_widget(block, self.area);
    }

    ///
    pub fn scroll_up(&mut self, num: i16) {
        self.position = std::cmp::max(0, self.position - num);
    }

    // TODO: 折返しによって発生する行数差分の計算方法が思いつかないため、思いついたら対応を追加する。(うまく取得が出来ない)
    ///
    pub fn scroll_down(&mut self, num: i16) {
        // get area data size
        let data_size = self.data.len() as i16;
        self.position = std::cmp::min(self.position + num, data_size - 1);
    }
}
