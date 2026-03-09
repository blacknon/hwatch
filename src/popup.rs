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
use unicode_width::UnicodeWidthStr;

use crate::common::centered_rect_with_size;

pub struct PopupWindow<'a> {
    title: String,
    text: Vec<Line<'a>>,
    area: Rect,
}

impl<'a> PopupWindow<'a> {
    pub fn new(title: impl Into<String>, text: Vec<String>) -> Self {
        let text = text
            .into_iter()
            .map(|s| Line::from(s)) // String -> Line<'static>
            .collect::<Vec<_>>();

        Self {
            title: title.into(),
            text,
            area: Rect::new(0, 0, 0, 0),
        }
    }

    /// draw popup window
    pub fn draw(&mut self, f: &mut Frame) {
        // title string
        let title_str = format!(" [{}] ", self.title);

        // measure inner size
        let (inner_w, inner_h) = measure_lines(&self.text);

        // Even if the content is empty, at least 1 line is reserved
        let inner_w = inner_w.max(1);
        let inner_h = inner_h.max(1);

        let title_min_w = UnicodeWidthStr::width(title_str.as_str()).max(6) as u16;
        let mut total_w = inner_w.saturating_add(2).max(title_min_w + 2);
        let mut total_h = inner_h.saturating_add(2);

        // 画面サイズに収まるようクランプ
        let max = f.area();
        total_w = total_w.min(max.width);
        total_h = total_h.min(max.height);

        // ---- 3) そのサイズで中央に配置 ----
        self.area = centered_rect_with_size(total_h, total_w, max);

        // ---- 4) 描画 ----
        let block = Paragraph::new(self.text.clone())
            .style(Style::default().bold())
            .block(
                Block::default()
                    .title(title_str)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().bold().fg(Color::Cyan)),
            )
            // wrap=false 前提の計測なので trim=false のまま
            .wrap(Wrap { trim: false });

        f.render_widget(Clear, self.area);
        f.render_widget(block, self.area);
    }
}

/// `Line` 配列から「内側幅(最長)・内側高さ(行数)」を返す
fn measure_lines(lines: &[Line]) -> (u16, u16) {
    let mut max_w: u16 = 0;
    for ln in lines {
        // Line::width() は Unicode 幅を考慮してくれる
        let w = ln.width() as u16;
        if w > max_w {
            max_w = w;
        }
    }
    let h = lines.len() as u16;
    (max_w, h)
}
