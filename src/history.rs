// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    symbols,
    widgets::{Block, Cell, Row, Table, TableState, Borders},
    Frame,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct History {
    pub timestamp: String,
    pub status: bool,
    pub num: u16,
}

pub struct HistoryArea {
    ///
    pub area: tui::layout::Rect,

    ///
    pub active: bool,

    ///
    data: Vec<Vec<History>>,

    ///
    state: TableState,
}

/// History Area Object Trait
impl HistoryArea {
    ///
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            active: false,
            data: vec![vec![History {
                timestamp: "latest                 ".to_string(),
                status: true,
                num: 0,
            }]],
            state: TableState::default(),
        }
    }

    ///
    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    ///
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    ///
    pub fn set_latest_status(&mut self, latest_status: bool) {
        self.data[0][0].status = latest_status;
    }

    ///
    pub fn update(&mut self, timestamp: String, status: bool, num: u16) {
        self.set_latest_status(status);

        // insert latest timestamp
        self.data.insert(
            1,
            vec![History {
                timestamp,
                status,
                num,
            }],
        );
    }

    ///
    pub fn reset_history_data(&mut self, data: Vec<Vec<History>>) {
        // @TODO: output mode切り替えでも使えるようにするため、indexを受け取るようにする

        // update data
        self.data = data;

        // set select num
        self.state.select(Some(0));
    }

    ///
    pub fn draw(&mut self, frame: &mut Frame) {
        // insert latest timestamp
        const LATEST_COLOR: Color = Color::Blue;
        let draw_data = &self.data;

        let rows = draw_data.iter().enumerate().map(|(ix, item)| {
            // set table height
            let height = item
                .iter()
                .map(|content| content.timestamp.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            // set cell data
            let cells = item.iter().map(|c| {
                let cell_style = Style::default().fg(match ix {
                    0 => LATEST_COLOR,
                    _ => match c.status {
                        true => Color::Green,
                        false => Color::Red,
                    },
                });
                Cell::from(Span::styled(c.timestamp.as_str(), cell_style))
            });

            Row::new(cells).height(height as u16)
        });

        let base_selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let selected_style = match self.active {
            true => match self.get_state_select() == 0 {
                true => base_selected_style.fg(LATEST_COLOR), // Necessary to make >> blue
                false => base_selected_style,
            },
            false => base_selected_style.fg(Color::DarkGray),
        };


        // debug
        // NOTE: 試しに枠で区切って様子見中。 最終的にはオプションでどうにかする。
        // let table_block = Block::default();

        // let top_right_border_set = symbols::border::Set {
        //     top_left: symbols::line::NORMAL.horizontal_down,
        //     ..symbols::border::PLAIN
        // };


        let table_block = Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)).border_set(symbols::border::PLAIN); // TEST
        // let table_block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White)); // TEST

        let table = Table::new(rows, [Constraint::Length(crate::HISTORY_WIDTH)])
            .block(table_block)
            .highlight_style(selected_style)
            .highlight_symbol(">>")
            .widths(&[Constraint::Percentage(100)]);

        frame.render_stateful_widget(table, self.area, &mut self.state);
    }

    ///
    pub fn get_history_size(&self) -> usize {
        self.data.len()
    }

    ///
    pub fn get_state_select(&self) -> usize {
        let i = match self.state.selected() {
            Some(i) => i,
            None => self.data.len() - 1,
        };

        if self.data.len() > i {
            return self.data[i][0].num as usize;
        } else {
            return 0;
        }
    }

    ///
    pub fn set_state_select(&mut self, index: usize) {
        // find index
        let mut i = 0;
        for d in self.data.iter() {
            if d[0].num == index as u16 {
                break;
            }
            i += 1;
        }

        self.state.select(Some(i));
    }

    ///
    pub fn next(&mut self, num: usize) {
        let i = match self.state.selected() {
            Some(i) =>{
            if i > num {
                    i - num
                } else {
                    0
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }

    ///
    pub fn previous(&mut self, num: usize) {
        let i= match self.state.selected() {
            Some(i) => {
                if i + num < self.data.len() - 1 {
                    i + num
                } else {
                    self.data.len() - 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }

    ///
    pub fn click_row(&mut self, row: u16) {
        let first_row = self.state.offset();
        let select_num = row as usize;
        if select_num < self.data.len() {
            self.state.select(Some(select_num + first_row));
        }
    }
}
