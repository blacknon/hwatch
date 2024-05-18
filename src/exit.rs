// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use ratatui::style::Stylize;
use tui::{
    layout::Rect,
    style::{Color, Style},
    prelude::Line,
    widgets::{Block, Borders, BorderType, Clear, Paragraph, Wrap},
    Frame,
};

use crate::common::centered_rect;

pub struct ExitWindow<'a> {
    ///
    text: Vec<Line<'a>>,

    ///
    area: Rect,
}

impl<'a> ExitWindow<'a> {
    pub fn new() -> Self {
        let text = vec![
            Line::from("Exit hwatch? (Y/N)"),
        ];

        Self {
            text,
            area: Rect::new(0, 0, 0, 0),
        }
    }

    ///
    pub fn draw(&mut self, f: &mut Frame) {
        let title = " [exit] ";

        let size = f.size();
        self.area = centered_rect(40, 10, size);

        // create block.
        let block = Paragraph::new(self.text.clone())
            .style(Style::default().bold())
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().bold().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(Clear, self.area);
        f.render_widget(block, self.area);
    }
}
