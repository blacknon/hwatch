// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::cmp;
use tui::{
    prelude::Line,
    style::{Color, Style},
    text::Span,
};

use hwatch_ansi as ansi;
use hwatch_diffmode::{gen_counter_str, DiffMode, DiffModeExt, DiffModeOptions, DifferenceType};

pub struct DiffModeAtWatch {
    header_width: usize,
    options: DiffModeOptions,
}

impl DiffModeAtWatch {
    pub fn new() -> Self {
        Self {
            header_width: 0,
            options: DiffModeOptions::new(),
        }
    }
}

impl DiffMode for DiffModeAtWatch {
    fn generate_watch_diff(&mut self, dest: &str, src: &str) -> Vec<Line<'static>> {
        //
        let mut result = Vec::new();

        // create dest text
        let mut text_dest_str = dest.to_string();
        let mut text_dest: String = "".to_string();
        for mut l in text_dest_str.lines() {
            if l.is_empty() {
                l = "\u{200B}";
            }

            text_dest.push_str(l);
            text_dest.push_str("\n");
        }

        // create src text
        let mut text_src_str = src.to_string();
        let mut text_src: String = "".to_string();
        for mut l in text_src_str.lines() {
            if l.is_empty() {
                l = "\u{200B}";
            }

            text_src.push_str(l);
            text_src.push_str("\n");
        }

        // create text vector
        let mut vec_src: Vec<&str> = text_src.lines().collect();
        let mut vec_dest: Vec<&str> = text_dest.lines().collect();

        // get max line
        let max_line = cmp::max(vec_src.len(), vec_dest.len());

        let mut counter = 1;
        let header_width = max_line.to_string().chars().count();

        // for diff lines
        let mut result = vec![];
        for i in 0..max_line {
            // push empty line
            if vec_src.len() <= i {
                vec_src.push("");
            }
            if vec_dest.len() <= i {
                vec_dest.push("");
            }

            let src_line = vec_src[i];
            let dest_line = vec_dest[i];

            let mut line_data = match self.options.get_color() {
                false => create_watch_diff_output_line(&src_line, &dest_line),
                true => create_watch_diff_output_line_with_ansi_for_watch(&src_line, &dest_line),
            };

            if self.options.get_line_number() {
                match line_data {
                    PrintElementData::Line(ref mut line) => {
                        line.spans.insert(
                            0,
                            Span::styled(
                                format!("{counter:>header_width$} | "),
                                Style::default().fg(Color::DarkGray),
                            ),
                        );
                    }
                    PrintElementData::String(ref mut line) => {
                        line.insert_str(
                            0,
                            &gen_counter_str(
                                self.is_color,
                                counter,
                                header_width,
                                DifferenceType::Same,
                            ),
                        );
                    }
                    PrintElementData::None() => {}
                }
            };

            result.push(line_data);
            counter += 1;
        }

        return expand_print_element_data(self.is_batch, result);

        return result;
    }

    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String> {
        //
        let mut result = Vec::new();

        return result;
    }

    fn set_option(&mut self, options: DiffModeOptions) {
        self.options = options;
    }
}

/// get_option の実装を DiffModeExt に分ける
impl DiffModeExt for DiffModeAtWatch {
    fn get_option<T: 'static>(&self) -> DiffModeOptions {
        self.options
    }

    fn get_header_width<T: 'static>(&self) -> usize {
        0
    }
}

// TODO: self関数を外だしして動かすように変更
// ----
// private function
// ----

///
fn create_watch_diff_output_line<'a>(
    &mut self,
    src_line: &str,
    dest_line: &str,
) -> PrintElementData<'a> {
    if src_line == dest_line {
        if self.is_batch {
            return PrintElementData::String(dest_line.to_string());
        } else {
            let line = Line::from(String::from(dest_line));
            return PrintElementData::Line(line);
        }
    }

    // Decompose lines by character.
    let mut src_chars: Vec<char> = src_line.chars().collect();
    let mut dest_chars: Vec<char> = dest_line.chars().collect();

    // 00a0 ... non-breaking space. watch mode only.
    // NOTE: Used because tui-rs skips regular space characters.
    let space: char = '\u{00a0}';

    // max char
    let max_char = cmp::max(src_chars.len(), dest_chars.len());

    let mut result_spans = vec![];
    let mut result_chars = vec![];

    let mut is_escape = false;
    let mut escape_code = "".to_string();
    for x in 0..max_char {
        if src_chars.len() <= x {
            src_chars.push(space);
        }

        if dest_chars.len() <= x {
            dest_chars.push(space);
        }

        let src_char = src_chars[x];
        let dest_char = dest_chars[x];

        if src_char != dest_char {
            // create spans
            let span_data = if dest_char == space {
                Span::from(' '.to_string())
            } else {
                Span::styled(
                    dest_chars[x].to_string(),
                    Style::default().add_modifier(Modifier::REVERSED),
                )
            };
            result_spans.push(span_data);

            // create chars
            let ansi_escape_sequence = '\x1b';
            let char_data = if dest_char == space {
                ' '
            } else {
                dest_chars[x]
            };

            if char_data == ansi_escape_sequence {
                escape_code = "".to_string();
                escape_code.push(char_data);
                is_escape = true;
            } else if is_escape {
                escape_code.push(char_data);
                if char_data == 'm' {
                    is_escape = false;
                }
                for c in escape_code.chars() {
                    result_chars.push(c);
                }
            } else {
                // let ansi_revese_style = ansi_term::Style::new().reverse();
                let ansi_reverse = format!("\x1b[7m{char_data}\x1b[7m");
                for c in ansi_reverse.chars() {
                    result_chars.push(c);
                }
            }
        } else {
            // create spans
            result_spans.push(Span::styled(dest_chars[x].to_string(), Style::default()));

            // create chars
            result_chars.push(dest_chars[x]);
        }
    }
    if self.is_batch {
        let mut data_str: String = result_chars.iter().collect();
        data_str.push_str("\x1b[0m");
        return PrintElementData::String(data_str);
    } else {
        return PrintElementData::Line(Line::from(result_spans));
    }
}

///
fn create_watch_diff_output_line_with_ansi_for_watch<'a>(
    &mut self,
    src_line: &str,
    dest_line: &str,
) -> PrintElementData<'a> {
    // If the contents are the same line.
    if src_line == dest_line {
        let new_spans = ansi::bytes_to_text(format!("{dest_line}\n").as_bytes());
        if let Some(spans) = new_spans.into_iter().next() {
            return PrintElementData::Line(spans);
        }
    }

    let src_colored_spans = gen_ansi_all_set_str(src_line);
    let dest_colored_spans = gen_ansi_all_set_str(dest_line);
    let mut src_spans = vec![];
    for mut src_span in src_colored_spans {
        src_spans.append(&mut src_span);
    }
    let mut dest_spans = vec![];
    for mut dest_span in dest_colored_spans {
        dest_spans.append(&mut dest_span);
    }

    // 00a0 ... non-breaking space.
    // NOTE: Used because tui-rs skips regular space characters.
    let space = '\u{00a0}'.to_string();
    let max_span = cmp::max(src_spans.len(), dest_spans.len());
    //
    let mut result = vec![];
    for x in 0..max_span {
        //
        if src_spans.len() <= x {
            src_spans.push(Span::from(space.to_string()));
        }

        //
        if dest_spans.len() <= x {
            dest_spans.push(Span::from(space.to_string()));
        }

        //
        if src_spans[x].content != dest_spans[x].content
            || src_spans[x].style != dest_spans[x].style
        {
            if dest_spans[x].content == space {
                let mut data = Span::from(' '.to_string());
                data.style = Style::default().add_modifier(Modifier::REVERSED);
                dest_spans[x] = data;
            } else {
                // add span
                dest_spans[x].style = dest_spans[x]
                    .style
                    .patch(Style::default().add_modifier(Modifier::REVERSED));
            }
        }

        result.push(dest_spans[x].clone());
    }

    result.push(Span::styled(space, Style::default()));

    return PrintElementData::Line(Line::from(result));
}
