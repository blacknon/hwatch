// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    prelude::Line,
    style::{Color, Style},
    text::Span,
};

use hwatch_ansi as ansi;
use hwatch_diffmode::{gen_counter_str, DiffMode, DiffModeExt, DiffModeOptions, DifferenceType};

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
        //
        let mut result = Vec::new();

        // NOTE: header_widthの計算をしておく
        // TODO: 2箇所で似たような処理をしているので、もうちょっとまとめたい
        let header_width: usize = if self.options.get_line_number() {
            dest.split('\n').count().to_string().chars().count() + 2
        } else {
            0
        };
        self.header_width = header_width.clone();
        let mut counter = 1;

        // split line
        for mut l in dest.split('\n') {
            if l.is_empty() {
                l = "\u{200B}";
            }

            let mut line = vec![];

            if self.options.get_line_number() {
                line.push(Span::styled(
                    format!("{counter:>header_width$} | "),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if self.options.get_color() {
                let data = ansi::bytes_to_text(format!("{l}\n").as_bytes());

                for d in data.lines {
                    line.extend(d.spans);
                }
            } else {
                line.push(Span::from(String::from(l)));
            }

            result.push(Line::from(line));
            counter += 1;
        }

        return result;
    }

    fn generate_batch_diff(&mut self, dest: &str, _: &str) -> Vec<String> {
        //
        let mut result = Vec::new();

        //
        let header_width: usize = if self.options.get_line_number() {
            dest.split('\n').count().to_string().chars().count() + 2
        } else {
            0
        };
        self.header_width = header_width.clone();
        let mut counter = 1;

        // split line
        for l in dest.split('\n') {
            let mut line = String::new();

            if self.options.get_line_number() {
                line.push_str(&gen_counter_str(
                    self.options.get_color(),
                    counter,
                    header_width,
                    DifferenceType::Same,
                ));
            }

            line.push_str(&l);
            result.push(line);

            counter += 1;
        }

        return result;
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
        self.header_width
    }
}
