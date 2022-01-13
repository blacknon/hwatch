// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
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

pub struct HistoryArea {
    ///
    area: tui::layout::Rect,

    ///
    data: Vec<String>,

    ///
    current: i32,

    ///
    scroll_position: u16,
}

/// History Area Object Trait
impl HistoryArea {
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            data: vec!["".to_string()],
            current: 0,
            scroll_position: 0,
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update(&mut self, history_data: &Vec<String>, current: i32) {}
}
