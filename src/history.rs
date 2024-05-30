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
use similar::{TextDiff, ChangeTag};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct History {
    // timestamp
    pub timestamp: String,

    // result status
    pub status: bool,

    // history number
    pub num: u16,

    // summary
    pub summary: HistorySummaryData,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HistorySummaryData {
    // output summary
    pub output: HistorySummary,

    // stdout summary
    pub stdout: HistorySummary,

    // stderr summary
    pub stderr: HistorySummary,
}

impl HistorySummaryData {
    pub fn init() -> Self {
        Self {
            output: HistorySummary::init(),
            stdout: HistorySummary::init(),
            stderr: HistorySummary::init(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HistorySummary {
    pub line_add: u16,
    pub line_rem: u16,
    pub char_add: u16,
    pub char_rem: u16,
}

impl HistorySummary {
    pub fn init() -> Self {
        Self {
            line_add: 0,
            line_rem: 0,
            char_add: 0,
            char_rem: 0,
        }
    }

    pub fn calc(&mut self, src: &str, dest: &str) {
        let line_diff = TextDiff::from_lines(src, dest);
        let char_diff = TextDiff::from_chars(src, dest);

        // line
        for op in line_diff.ops().iter() {
            for change in line_diff.iter_inline_changes(op) {
                match change.tag() {
                    ChangeTag::Insert => {self.line_add += 1},
                    ChangeTag::Delete => {self.line_rem += 1},
                    _ => {},
                }
            }
        }

        // char
        for op in char_diff.ops().iter() {
            for change in char_diff.iter_inline_changes(op) {
                match change.tag() {
                    ChangeTag::Insert => {self.char_add += 1},
                    ChangeTag::Delete => {self.char_rem += 1},
                    _ => {},
                }
            }
        }
    }
}

enum HistorySummaryType {
    None,
    Line,
    Char,
}

pub struct HistoryArea {
    ///
    pub area: tui::layout::Rect,

    ///
    pub active: bool,

    /// View data

    /// History data.
    ///
    data: Vec<Vec<History>>,

    /// State information including the selected position
    state: TableState,

    /// Set summary display mode.
    summary: HistorySummaryType,

    /// is enable border
    border: bool,

    /// is hide header
    hide_header: bool,

    /// is enable scroll bar
    scroll_bar: bool,
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
                summary: HistorySummaryData::init(),
            }]],
            state: TableState::default(),
            summary: HistorySummaryType::None,
            border: false,
            hide_header: false,
            scroll_bar: false,
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
    pub fn set_border(&mut self, border: bool) {
        self.border = border;
    }

    ///
    pub fn set_scroll_bar(&mut self, scroll_bar: bool) {
        self.scroll_bar = scroll_bar;
    }

    ///
    pub fn set_hide_header(&mut self, hide_header: bool) {
        self.hide_header = hide_header;
    }

    ///
    pub fn update(&mut self, timestamp: String, status: bool, num: u16) {
        // set result statu to latest
        self.set_latest_status(status);

        // insert latest timestamp
        self.data.insert(
            1,
            vec![History {
                timestamp,
                status,
                num,
                summary: HistorySummaryData::init(),
            }],
        );
    }

    ///
    /// TODO: まいどリセット時にhistoryの再生成かけてたけど、これだとsummaryも再対応しないといけないのでメモリ内にオリジナルを保持させて、それを加工する方向に変更する
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

        let pane_block: Block<'_>;
        let history_width: u16;
        if self.border {
            history_width = crate::HISTORY_WIDTH + 1;
            if self.hide_header {
                pane_block = Block::default();
            } else {
                pane_block = Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_set(
                        symbols::border::Set {
                            top_left: symbols::line::NORMAL.horizontal_down,
                            ..symbols::border::PLAIN
                        }
                    );
            }
        } else {
            history_width = crate::HISTORY_WIDTH;
            pane_block = Block::default()
        }

        let table = Table::new(rows, [Constraint::Length(history_width)])
            .block(pane_block)
            .highlight_style(selected_style)
            .highlight_symbol(">>")
            .widths(&[Constraint::Percentage(100)]);

        // render table
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
