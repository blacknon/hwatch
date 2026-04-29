// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::cmp;
use tui::{
    prelude::Line,
    style::{Color, Modifier, Style},
    text::Span,
};

use hwatch_ansi as ansi;
use hwatch_ansi::gen_ansi_all_set_str;
use hwatch_diffmode::{
    render_diff_rows_as_batch, render_diff_rows_as_watch, text_eq_ignoring_space_blocks, DiffMode,
    DiffModeExt, DiffModeOptions, DiffRow, DifferenceType,
};

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
        let (header_width, rows) = generate_watch_diff_rows(dest, src, &self.options);
        self.header_width = header_width;
        render_diff_rows_as_watch(rows, self.options.get_line_number(), header_width)
    }

    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String> {
        let (header_width, rows) = generate_watch_diff_rows(dest, src, &self.options);
        self.header_width = header_width;
        render_diff_rows_as_batch(
            rows,
            self.options.get_color(),
            self.options.get_line_number(),
            header_width,
        )
    }

    fn get_header_text(&self) -> String {
        return String::from("Watch");
    }

    fn get_support_only_diffline(&self) -> bool {
        return false;
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
        self.header_width + 3
    }
}

// ----
// private function
// ----

fn generate_watch_diff_rows<'a>(
    dest: &str,
    src: &str,
    options: &DiffModeOptions,
) -> (usize, Vec<DiffRow<'a>>) {
    let mut result = Vec::new();

    // create dest text
    let text_dest_str = dest.to_string();
    let mut text_dest: String = "".to_string();
    for mut l in text_dest_str.lines() {
        if l.is_empty() {
            l = "\u{200B}";
        }

        text_dest.push_str(l);
        text_dest.push_str("\n");
    }

    // create src text
    let text_src_str = src.to_string();
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

    let header_width = max_line.to_string().chars().count();

    // for diff lines
    for (counter, i) in (1..).zip(0..max_line) {
        // push empty line
        if vec_src.len() <= i {
            vec_src.push("");
        }
        if vec_dest.len() <= i {
            vec_dest.push("");
        }

        let src_line = vec_src[i];
        let dest_line = vec_dest[i];

        let watch_line = match options.get_color() {
            false => create_watch_diff_output_line(
                &dest_line,
                &src_line,
                options.get_ignore_spaceblock(),
            ),
            true => create_watch_diff_output_line_with_ansi_for_watch(
                &dest_line,
                &src_line,
                options.get_ignore_spaceblock(),
            ),
        };
        let batch_line = match options.get_color() {
            false => create_watch_diff_output_line_for_batch(
                &dest_line,
                &src_line,
                options.get_ignore_spaceblock(),
            ),
            true => create_watch_diff_output_line_with_ansi_for_batch(
                &dest_line,
                &src_line,
                options.get_ignore_spaceblock(),
            ),
        };

        result.push(DiffRow {
            watch_line,
            batch_line,
            line_number: Some(counter),
            diff_type: DifferenceType::Same,
        });
    }

    (header_width, result)
}

///
fn create_watch_diff_output_line<'a>(
    dest_line: &str,
    src_line: &str,
    ignore_spaceblock: bool,
) -> Line<'a> {
    if text_eq_ignoring_space_blocks(src_line, dest_line, ignore_spaceblock) {
        return Line::from(String::from(dest_line));
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
    let _ = result_chars;
    Line::from(result_spans)
}

fn create_watch_diff_output_line_for_batch(
    dest_line: &str,
    src_line: &str,
    ignore_spaceblock: bool,
) -> String {
    if text_eq_ignoring_space_blocks(src_line, dest_line, ignore_spaceblock) {
        return dest_line.to_string();
    }

    let mut src_chars: Vec<char> = src_line.chars().collect();
    let mut dest_chars: Vec<char> = dest_line.chars().collect();
    let space: char = '\u{00a0}';
    let max_char = cmp::max(src_chars.len(), dest_chars.len());
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
                let ansi_reverse = format!("\x1b[7m{char_data}\x1b[7m");
                for c in ansi_reverse.chars() {
                    result_chars.push(c);
                }
            }
        } else {
            result_chars.push(dest_chars[x]);
        }
    }

    let mut data_str: String = result_chars.iter().collect();
    data_str.push_str("\x1b[0m");
    data_str
}

///
fn create_watch_diff_output_line_with_ansi_for_watch<'a>(
    dest_line: &str,
    src_line: &str,
    ignore_spaceblock: bool,
) -> Line<'a> {
    // If the contents are the same line.
    if text_eq_ignoring_space_blocks(src_line, dest_line, ignore_spaceblock) {
        let new_spans = ansi::bytes_to_text(format!("{dest_line}\n").as_bytes());
        if let Some(spans) = new_spans.into_iter().next() {
            return spans;
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

    Line::from(result)
}

fn create_watch_diff_output_line_with_ansi_for_batch(
    dest_line: &str,
    src_line: &str,
    ignore_spaceblock: bool,
) -> String {
    if text_eq_ignoring_space_blocks(src_line, dest_line, ignore_spaceblock) {
        return dest_line.to_string();
    }

    let mut rendered = String::new();
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

    let space = '\u{00a0}'.to_string();
    let max_span = cmp::max(src_spans.len(), dest_spans.len());
    for x in 0..max_span {
        if src_spans.len() <= x {
            src_spans.push(Span::from(space.to_string()));
        }
        if dest_spans.len() <= x {
            dest_spans.push(Span::from(space.to_string()));
        }

        let mut span = dest_spans[x].clone();
        if src_spans[x].content != dest_spans[x].content
            || src_spans[x].style != dest_spans[x].style
        {
            if dest_spans[x].content == space {
                span = Span::raw(" ");
            } else {
                span.style = span
                    .style
                    .patch(Style::default().add_modifier(Modifier::REVERSED));
            }
        }

        let ansi_style = ansi_span_to_style(&span.style);
        let content = if span.content == space {
            " ".to_string()
        } else {
            span.content.into_owned()
        };
        rendered.push_str(&ansi_style.paint(content).to_string());
    }

    rendered.push_str("\x1b[0m");
    rendered
}

fn ansi_span_to_style(style: &Style) -> ansi_term::Style {
    let mut ansi_style = ansi_term::Style::new();

    if let Some(fg) = style.fg.and_then(tui_color_to_ansi) {
        ansi_style = ansi_style.fg(fg);
    }
    if let Some(bg) = style.bg.and_then(tui_color_to_ansi) {
        ansi_style = ansi_style.on(bg);
    }
    if style.add_modifier.contains(Modifier::BOLD) {
        ansi_style = ansi_style.bold();
    }
    if style.add_modifier.contains(Modifier::DIM) {
        ansi_style = ansi_style.dimmed();
    }
    if style.add_modifier.contains(Modifier::ITALIC) {
        ansi_style = ansi_style.italic();
    }
    if style.add_modifier.contains(Modifier::UNDERLINED) {
        ansi_style = ansi_style.underline();
    }
    if style.add_modifier.contains(Modifier::REVERSED) {
        ansi_style = ansi_style.reverse();
    }

    ansi_style
}

fn tui_color_to_ansi(color: Color) -> Option<ansi_term::Colour> {
    match color {
        Color::Black => Some(ansi_term::Colour::Black),
        Color::Red => Some(ansi_term::Colour::Red),
        Color::Green => Some(ansi_term::Colour::Green),
        Color::Yellow => Some(ansi_term::Colour::Yellow),
        Color::Blue => Some(ansi_term::Colour::Blue),
        Color::Magenta => Some(ansi_term::Colour::Purple),
        Color::Cyan => Some(ansi_term::Colour::Cyan),
        Color::Gray => Some(ansi_term::Colour::White),
        Color::DarkGray => Some(ansi_term::Colour::Fixed(8)),
        Color::LightRed => Some(ansi_term::Colour::Fixed(9)),
        Color::LightGreen => Some(ansi_term::Colour::Fixed(10)),
        Color::LightYellow => Some(ansi_term::Colour::Fixed(11)),
        Color::LightBlue => Some(ansi_term::Colour::Fixed(12)),
        Color::LightMagenta => Some(ansi_term::Colour::Fixed(13)),
        Color::LightCyan => Some(ansi_term::Colour::Fixed(14)),
        Color::White => Some(ansi_term::Colour::White),
        Color::Rgb(r, g, b) => Some(ansi_term::Colour::RGB(r, g, b)),
        Color::Indexed(index) => Some(ansi_term::Colour::Fixed(index)),
        Color::Reset => None,
    }
}
