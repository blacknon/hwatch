// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame, Terminal,
};

pub struct WatchArea<'a> {
    ///
    area: tui::layout::Rect,

    ///
    data: Vec<Spans<'a>>,

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

    pub fn update_output(&mut self, text: &str) {
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

    pub fn update_output_diff(&mut self, text1: &str, text2: &str) {}

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
