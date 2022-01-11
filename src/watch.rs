// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame, Terminal,
};

pub struct WatchArea {
    area: tui::layout::Rect,
    text: String,
    scroll_position: u16,
}

/// Signal Trait
impl WatchArea {
    pub fn new() -> Self {
        //! new WatchArea
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            text: "".to_string(),
            scroll_position: 0,
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn draw<B: Backend>(&mut self, frame: Frame<B>) {
        let block = Paragraph::new(self.text.clone());

        frame.render_widget(block, self.area);
    }
}
