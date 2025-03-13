// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use ratatui::text::Span;
use tui::{
    layout::Rect,
    style::{Color, Style},
    prelude::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::keymap::{get_input_action_description, InputAction, Keymap};
use crate::common::centered_rect;

pub struct KeyData {
    pub key: String,
    pub description: String,
    pub action: InputAction,
}

pub struct HelpWindow<'a> {
    ///
    text: Vec<Line<'a>>,

    ///
    area: Rect,

    ///
    position: i16,

    ///
    lines: i16,
}

/// History Area Object Trait
impl<'a> HelpWindow<'a> {
    pub fn new(keymap: Keymap) -> Self {
        let text = gen_help_text(keymap);

        Self {
            text,
            area: Rect::new(0, 0, 0, 0),
            position: 0,
            lines: 0,
        }
    }

    ///
    pub fn draw(&mut self, f: &mut Frame) {
        let title = " [help] ";

        let size = f.area();
        self.area = centered_rect(80, 70, size);

        let width = self.area.width;

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

        self.lines = block.line_count(width) as i16;

        f.render_widget(Clear, self.area);
        f.render_widget(block, self.area);
    }

    ///
    pub fn scroll_up(&mut self, num: i16) {
        self.position = std::cmp::max(0, self.position - num);
    }

    ///
    pub fn scroll_down(&mut self, num: i16) {
        let height: u16 = self.area.height - 2; // top/bottom border = 2
        if self.lines > height as i16 {
            self.position = std::cmp::min(self.position + num, self.lines - height as i16);
        }
    }

    pub fn page_up(&mut self) {
        let height: u16 = self.area.height - 2; // top/bottom border = 2
        if self.lines > height as i16 {
            self.position = std::cmp::max(0, self.position - height as i16);
        }
    }

    pub fn page_down(&mut self) {
        let height: u16 = self.area.height - 2; // top/bottom border = 2
        if self.lines > height as i16 {
            self.position = std::cmp::min(self.position + height as i16, self.lines - height as i16);
        }
    }

    pub fn scroll_top(&mut self) {
        self.position = 0;
    }

    pub fn scroll_end(&mut self) {
        let height: u16 = self.area.height - 2; // top/bottom border = 2
        if self.lines > height as i16 {
            self.position = self.lines - height as i16;
        }
    }


}

fn gen_help_text_from_key_data<'a>(data: Vec<KeyData>) -> Vec<Line<'a>> {
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
fn gen_help_text<'a>(keymap: Keymap) -> Vec<Line<'a>> {
    let mut keydata_list = vec![];

    for (_, input_event_content) in &keymap {
        let key = input_event_content.input.to_str();
        let description = get_input_action_description(input_event_content.action);

        keydata_list.push(KeyData { key: key, description: description, action: input_event_content.action});
    };

    // sort
    keydata_list.sort_by(|a, b| a.action.cmp(&b.action));

    let text = gen_help_text_from_key_data(keydata_list);

    text
}
