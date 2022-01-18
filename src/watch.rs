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
    color: bool,

    ///
    state: u16,
}

/// Watch Area Object Trait
impl<'a> WatchArea<'a> {
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            data: vec![Spans::from("")],
            color: false,
            state: 0,
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update_output(&mut self, text: &str) {
        // init self.data
        self.data = vec![];
        let lines = text.split("\n");

        for l in lines {
            self.data.push(Spans::from(String::from(l)));
        }
    }

    pub fn update_output_diff(&mut self, text1: &str, text2: &str) {}

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let block = Paragraph::new(self.data.clone()).wrap(Wrap { trim: true });
        frame.render_widget(block, self.area);
    }

    pub fn input(&mut self, event: crossterm::event::Event) {}

    pub fn scroll_up(&mut self, num: u16) {}

    pub fn scroll_down(&mut self, num: u16) {}
}
