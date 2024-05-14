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
use crate::keymap::{Keymap, get_input_action_description};

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
    pub fn new(keymap: Keymap) -> Self {
        let text = gen_help_text(keymap);

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
// TODO: keymapから読み取らせる方式とする(`description`をどのように取得するか・どこに説明を書くかは要検討)
fn gen_help_text<'a>(keymap: Keymap) -> Vec<Line<'a>> {
    let mut keydata_list = vec![];

    for (_, input_event_content) in &keymap {
        let key = input_event_content.key.to_str();
        let description = get_input_action_description(input_event_content.action);

        keydata_list.push(KeyData { key: key, description: description });
    };

    // sort
    keydata_list.sort_by(|a, b| a.key.cmp(&b.key));

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
