// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use ratatui::style::Stylize;
use tui::{
    layout::Rect,
    prelude::Line,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::common::centered_rect_with_size;

pub struct ExitWindow<'a> {
    ///
    text: Vec<Line<'a>>,

    ///
    area: Rect,
}

impl ExitWindow<'_> {
    pub fn new() -> Self {
        let text = vec![
            Line::from(" Exit hwatch?"),
            Line::from("   Press 'Y' or 'Q'  : Quit."),
            Line::from("   Press 'N' or 'Esc': Stay."),
        ];

        Self {
            text,
            area: Rect::new(0, 0, 0, 0),
        }
    }

    ///
    pub fn draw(&mut self, f: &mut Frame) {
        let title = " [exit] ";

        // TODO: 枠を含めて3行にする
        let size = f.area();
        self.area = centered_rect_with_size(5, 32, size);

        // create block.
        let block = Paragraph::new(self.text.clone())
            .style(Style::default().bold())
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().bold().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(Clear, self.area);
        f.render_widget(block, self.area);
    }
}
