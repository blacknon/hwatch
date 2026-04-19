// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    prelude::Line,
    style::{Color, Style},
    text::Span,
};

use hwatch_ansi as ansi;
use hwatch_diffmode::{
    render_diff_rows_as_batch, render_diff_rows_as_watch, DiffMode, DiffModeExt, DiffModeOptions,
    DiffRow,
};

pub struct DiffModeAtPlane {
    header_width: usize,
    options: DiffModeOptions,
}

impl DiffModeAtPlane {
    pub fn new() -> Self {
        Self {
            header_width: 0,
            options: DiffModeOptions::new(),
        }
    }
}

impl DiffMode for DiffModeAtPlane {
    fn generate_watch_diff(&mut self, dest: &str, _: &str) -> Vec<Line<'static>> {
        let (header_width, rows) = generate_plane_rows(dest, self.options.get_color());
        self.header_width = header_width.clone();
        render_diff_rows_as_watch(rows, self.options.get_line_number(), header_width)
    }

    fn generate_batch_diff(&mut self, dest: &str, _: &str) -> Vec<String> {
        let (header_width, rows) = generate_plane_rows(dest, self.options.get_color());
        self.header_width = header_width.clone();
        render_diff_rows_as_batch(
            rows,
            self.options.get_color(),
            self.options.get_line_number(),
            header_width,
        )
    }

    fn get_header_text(&self) -> String {
        return String::from("None");
    }

    fn get_support_only_diffline(&self) -> bool {
        return false;
    }

    fn set_option(&mut self, options: DiffModeOptions) {
        self.options = options;
    }
}

/// get_option の実装を DiffModeExt に分ける
impl DiffModeExt for DiffModeAtPlane {
    fn get_option<T: 'static>(&self) -> DiffModeOptions {
        self.options
    }

    fn get_header_width<T: 'static>(&self) -> usize {
        self.header_width + 3
    }
}

fn generate_plane_rows<'a>(dest: &str, is_color: bool) -> (usize, Vec<DiffRow<'a>>) {
    let header_width = dest.split('\n').count().to_string().chars().count();
    let mut rows = Vec::new();
    let mut counter = 1;

    for mut l in dest.split('\n') {
        if l.is_empty() {
            l = "\u{200B}";
        }

        let watch_line = if is_color {
            let mut spans = Vec::new();
            let data = ansi::bytes_to_text(format!("{l}\n").as_bytes());
            for d in data.lines {
                spans.extend(d.spans);
            }
            Line::from(spans)
        } else {
            Line::from(vec![Span::styled(
                String::from(l),
                Style::default().fg(Color::Reset),
            )])
        };

        rows.push(DiffRow {
            watch_line,
            batch_line: l.to_string(),
            line_number: Some(counter),
            diff_type: hwatch_diffmode::DifferenceType::Same,
        });
        counter += 1;
    }

    (header_width, rows)
}
