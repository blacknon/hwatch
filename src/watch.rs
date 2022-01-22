// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{backend::Backend, style::Style, text::Spans, widgets::Paragraph, Frame};

// local module
use diff;
use view::DiffMode;

pub struct WatchArea<'a> {
    ///
    area: tui::layout::Rect,

    ///
    pub data: Vec<Spans<'a>>,

    ///
    ansi_color: bool,

    ///
    position: i16,
}

/// Watch Area Object Trait
impl<'a> WatchArea<'a> {
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),

            data: vec![Spans::from("")],

            ansi_color: false,
            position: 0,
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update_output(&mut self, text: String) {
        // init self.data
        self.data = vec![];

        match self.ansi_color {
            true => {
                let data = ansi4tui::bytes_to_text(text.as_bytes().to_vec());
                self.data = data.lines;
            }

            false => {
                let lines = text.split("\n");
                for l in lines {
                    self.data.push(Spans::from(String::from(l)));
                }
            }
        }
    }

    pub fn update_output_diff<'watch_data>(
        &'watch_data mut self,
        diff_mode: DiffMode,
        text1: &'a str,
        text2: &'a str,
    ) {
        let mut data = vec![];

        // get diffrense str
        match diff_mode {
            DiffMode::Watch => {
                data = diff::get_watch_diff(text1, text2);
            }

            _ => {}
        }

        //init self.data
        self.data = data;
    }

    pub fn set_ansi_color(&mut self, ansi_color: bool) {
        self.ansi_color = ansi_color;
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let block = Paragraph::new(self.data.clone())
            .style(Style::default())
            .scroll((self.position as u16, 0));
        frame.render_widget(block, self.area);
    }

    pub fn input(&mut self, event: crossterm::event::Event) {}

    pub fn scroll_up(&mut self, num: i16) {
        if 0 <= self.position - num {
            self.position = self.position - num
        }
    }

    pub fn scroll_down(&mut self, num: i16) {
        if self.data.len() as i16 > self.position + num {
            self.position = self.position + num
        }
    }
}
