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
        Paragraph, Row, Sparkline, Table, TableState, Tabs, Wrap,
    },
    Frame, Terminal,
};

pub struct HistoryArea {
    ///
    area: tui::layout::Rect,

    ///
    data: Vec<Vec<String>>,

    ///
    state: TableState,
}

/// History Area Object Trait
impl HistoryArea {
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            data: vec![vec!["latest".to_string()]],
            state: TableState::default(),
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update(&mut self, timestamp: String) {
        // insert latest timestamp
        self.data.insert(1, vec![timestamp]);
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        // insert latest timestamp
        let draw_data = &self.data;

        // style
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let rows = draw_data.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(height as u16)
        });

        let table = Table::new(rows)
            .block(Block::default())
            .highlight_style(selected_style)
            .highlight_symbol("> ")
            .widths(&[Constraint::Percentage(100)]);

        frame.render_stateful_widget(table, self.area, &mut self.state);
    }

    pub fn get_state_select(&mut self) -> usize {
        let i = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };
        return i;
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.data.len() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
