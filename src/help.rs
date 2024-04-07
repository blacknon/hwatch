// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): keyのhelpをテキストからテーブルにする?
// TODO(blacknon): keyの内容をカスタムし、それをhelpで出せるようにする
// TODO(blacknon): keyの内容をvecで渡してやるようにする
// TODO(blacknon): keyの内容を折り返して表示させるようにする

use ratatui::text::Span;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    prelude::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::keys::{self, KeyData};

pub struct HelpWindow<'a> {
    ///
    text: Vec<Line<'a>>,

    ///
    area: Rect,

    ///
    position: i16,
}

/// History Area Object Trait
impl<'a> HelpWindow<'a> {
    pub fn new() -> Self {
        let text = gen_help_text();

        Self {
            text,
            area: Rect::new(0, 0, 0, 0),
            position: 0
        }
    }

    ///
    pub fn draw(&mut self, f: &mut Frame) {
        let title = " [help] ";

        let size = f.size();
        let area = centered_rect(80, 70, size);

        // create block.
        let block = Paragraph::new(self.text.clone())
            .style(Style::default().fg(Color::LightGreen))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray).bg(Color::Reset)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.position as u16, 0));

        f.render_widget(Clear, area);
        f.render_widget(block, area);
    }

    ///
    pub fn scroll_up(&mut self, num: i16) {
        self.position = std::cmp::max(0, self.position - num);
    }

    ///
    pub fn scroll_down(&mut self, num: i16) {
        // get area data size
        let data_size = self.text.len() as i16;

        if data_size > self.position + num {
            self.position += num
        }

        if self.text.len() as i16 > self.area.height as i16 {
            self.position = std::cmp::min(self.position + num, self.text.len() as i16 - self.area.height as i16);
        }
    }
}

fn gen_help_text_from_key_data<'a>(data: Vec<keys::KeyData>) -> Vec<Line<'a>> {
    let mut text = vec![];

    for key_data in data {
        let line1 = Line::from(vec![
            Span::styled(
                " - [",
                Style::default().fg(Color::LightGreen),
            ),
            Span::styled(
                key_data.key,
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "] key :",
                Style::default().fg(Color::LightGreen),
            )
            ]
        );

        let line2 = Line::from(vec![
            Span::styled(
                "         ",
                Style::default(),
            ),
            Span::styled(
                key_data.description,
                Style::default().fg(Color::White),
            ),
            ]
        );

        text.push(line1);
        text.push(line2);
    }

    text
}

///
fn gen_help_text<'a>() -> Vec<Line<'a>> {
    let keydata_list = vec![
        KeyData { key: "h".to_string(), description: "show this help message.".to_string() },
        // toggle
        KeyData { key: "c".to_string(), description: "toggle color mode.".to_string() },
        KeyData { key: "n".to_string(), description: "toggle line number.".to_string() },
        KeyData { key: "d".to_string(), description: "switch diff mode at None, Watch, Line, and Word mode.".to_string() },
        KeyData { key: "t".to_string(), description: "toggle ui (history pane & header both on/off).".to_string() },
        KeyData { key: "Bkspace".to_string(), description: "toggle history pane.".to_string() },
        KeyData { key: "m".to_string(), description: "toggle mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.".to_string() },
        // exit hwatch
        KeyData { key: "q".to_string(), description: "exit hwatch.".to_string() },
        // change diff
        KeyData { key: "0".to_string(), description: "disable diff.".to_string() },
        KeyData { key: "1".to_string(), description: "switch Watch type diff.".to_string() },
        KeyData { key: "2".to_string(), description: "switch Line type diff.".to_string() },
        KeyData { key: "3".to_string(), description: "switch Word type diff.".to_string() },
        // change output
        KeyData { key: "F1".to_string(), description: "change output mode as stdout.".to_string() },
        KeyData { key: "F2".to_string(), description: "change output mode as stderr.".to_string() },
        KeyData { key: "F3".to_string(), description: "change output mode as output(stdout/stderr set.).".to_string() },
        // change interval
        KeyData { key: "+".to_string(), description: "Increase interval by .5 seconds.".to_string() },
        KeyData { key: "-".to_string(), description: "Decrease interval by .5 seconds.".to_string() },
        // change use area
        KeyData { key: "Tab".to_string(), description: "toggle current area at history or watch.".to_string() },
        // filter text input
        KeyData { key: "/".to_string(), description: "filter history by string.".to_string() },
        KeyData { key: "*".to_string(), description: "filter history by regex.".to_string() },
        KeyData { key: "ESC".to_string(), description: "unfiltering.".to_string() },
    ];

    let text = gen_help_text_from_key_data(keydata_list);

    text
}

///
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
