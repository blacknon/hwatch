// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    backend::{Backend, CrosstermBackend},
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

use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

pub struct WatchArea<'a> {
    area: tui::layout::Rect,
    data: Vec<Spans<'a>>,
    scroll_position: u16,
}

/// Signal Trait
impl<'a> WatchArea<'a> {
    pub fn new() -> Self {
        //! new WatchArea
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            data: vec![Spans::from("")],
            scroll_position: 0,
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update_data(&mut self, text: &str) {
        // init self.data
        self.data = vec![Spans::from("")];
        let lines = text.split("\n");

        for l in lines {
            self.data.push(Spans::from(String::from(l)));
        }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let block = Paragraph::new(self.data.clone()).wrap(Wrap { trim: true });
        frame.render_widget(block, self.area);
    }
}
