// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    backend::Backend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Cell, Row, Table, TableState},
    Frame,
};

#[derive(Clone)]
struct History {
    timestamp: String,
    status: bool,
}

pub struct HistoryArea {
    ///
    pub area: tui::layout::Rect,

    ///
    data: Vec<Vec<History>>,

    ///
    state: TableState,
}

/// History Area Object Trait
impl HistoryArea {
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            data: vec![vec![History {
                timestamp: "latest                 ".to_string(),
                status: true,
            }]],
            state: TableState::default(),
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn set_latest_status(&mut self, latest_status: bool) {
        self.data[0][0].status = latest_status;
    }

    pub fn update(&mut self, timestamp: String, status: bool) {
        self.set_latest_status(status);

        // insert latest timestamp
        self.data.insert(
            1,
            vec![History {
                timestamp: timestamp,
                status: status,
            }],
        );
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        // insert latest timestamp
        let draw_data = &self.data;

        // style
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let rows = draw_data.iter().map(|item| {
            // set table height
            let height = item
                .iter()
                .map(|content| content.timestamp.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            // set cell data
            let cells = item.iter().map(|c| {
                let cell_style: Style;
                match c.status {
                    true => cell_style = Style::default().fg(Color::Green),
                    false => cell_style = Style::default().fg(Color::Red),
                }
                Cell::from(Span::styled(c.timestamp.as_str(), cell_style))
            });

            Row::new(cells).height(height as u16)
        });

        let table = Table::new(rows)
            .block(Block::default())
            .highlight_style(selected_style)
            .highlight_symbol(">>")
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

    // NOTE: TODO:
    // It will not be supported until the following issues are resolved.
    //     - https://github.com/fdehau/tui-rs/issues/495
    //
    // pub fn click_row(&mut self, row: u16) {
    //     let select_num = row as usize;
    //     self.state.select(Some(select_num));
    // }
}
