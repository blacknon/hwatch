// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[warn(unused)]
use tui::{prelude::Line, style::Style, text::Span};

use hwatch_ansi as ansi;
use hwatch_diffmode::{
    render_diff_rows_as_batch, render_diff_rows_as_watch, DiffMode, DiffModeExt, DiffModeOptions,
    DiffRow, DifferenceType,
};
use similar::{ChangeTag, InlineChange, TextDiff};
use std::cmp;

pub struct DiffModeAtLineDiff {
    header_width: usize,
    pub is_word_highlight: bool,
    options: DiffModeOptions,
}

impl DiffModeAtLineDiff {
    pub fn new() -> Self {
        Self {
            header_width: 3,
            is_word_highlight: false,
            options: DiffModeOptions::new(),
        }
    }
}

impl DiffMode for DiffModeAtLineDiff {
    fn generate_watch_diff(&mut self, dest: &str, src: &str) -> Vec<Line<'static>> {
        let (header_width, rows) =
            gen_line_diff_rows(dest, src, self.is_word_highlight, &self.options);
        self.header_width = header_width;
        render_diff_rows_as_watch(rows, self.options.get_line_number(), header_width)
    }

    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String> {
        let (header_width, rows) =
            gen_line_diff_rows(dest, src, self.is_word_highlight, &self.options);
        self.header_width = header_width;
        render_diff_rows_as_batch(
            rows,
            self.options.get_color(),
            self.options.get_line_number(),
            header_width,
        )
    }

    fn get_header_text(&self) -> String {
        let header_text = match (self.is_word_highlight, self.options.get_only_diffline()) {
            (true, true) => "Word(Only)",
            (true, false) => "Word      ",
            (false, true) => "Line(Only)",
            (false, false) => "Line      ",
        };
        return String::from(header_text);
    }

    fn get_support_only_diffline(&self) -> bool {
        return true;
    }

    fn set_option(&mut self, options: DiffModeOptions) {
        self.options = options;
    }
}

/// get_option の実装を DiffModeExt に分ける
impl DiffModeExt for DiffModeAtLineDiff {
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

// TODO: options系はstructで渡すようにする
fn gen_line_diff_rows<'a>(
    dest: &str,
    src: &str,
    is_word_highlight: bool,
    options: &DiffModeOptions,
) -> (usize, Vec<DiffRow<'a>>) {
    let text_dest = dest.to_string();
    let text_dest_bytes = text_dest.as_bytes().to_vec();

    // tab expand src
    let text_src = src.to_string();
    let text_src_bytes = text_src.as_bytes().to_vec();

    // Create diff data
    let diff_set = TextDiff::from_lines(&text_src_bytes, &text_dest_bytes);

    // src and dest text's line count.
    let src_len = diff_set.old_slices().len();
    let dest_len = diff_set.new_slices().len();

    // get line_number width
    let header_width = cmp::max(src_len, dest_len).to_string().chars().count();

    // create result
    let mut rows = vec![];
    for op in diff_set.ops().iter() {
        for change in diff_set.iter_inline_changes(op) {
            if let Some(row) = gen_line_diff_row(&change, is_word_highlight, options) {
                rows.push(row);
            }
        }
    }
    (header_width, rows)
}

fn gen_equal_row_from_line<'a>(
    line: &str,
    line_index: usize,
    options: &DiffModeOptions,
) -> Option<DiffRow<'a>> {
    if options.get_only_diffline() {
        return None;
    }

    let batch_line = if options.get_color() {
        line.trim_end_matches('\n').to_string()
    } else {
        ansi::get_ansi_strip_str(line)
            .trim_end_matches('\n')
            .to_string()
    };

    let watch_line = if options.get_color() {
        let mut spans = vec![Span::from("   ")];
        let colored_data = ansi::bytes_to_text(line.as_bytes());
        for d in colored_data.lines {
            for span in d.spans {
                spans.push(span);
            }
        }
        Line::from(spans)
    } else {
        Line::from(vec![
            Span::from("   "),
            Span::styled(line.to_string(), Style::default()),
        ])
    };

    Some(DiffRow {
        watch_line,
        batch_line,
        line_number: Some(line_index + 1),
        diff_type: DifferenceType::Same,
    })
}

fn gen_simple_line_diff_row<'a>(
    tag: ChangeTag,
    line: &str,
    line_index: usize,
    options: &DiffModeOptions,
) -> Option<DiffRow<'a>> {
    let (line_header, diff_type, tui_line_style, str_line_style) = match tag {
        ChangeTag::Delete => (
            "-  ",
            DifferenceType::Rem,
            Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_REM),
            ansi_term::Style::new().fg(hwatch_diffmode::COLOR_BATCH_LINE_REM),
        ),
        ChangeTag::Insert => (
            "+  ",
            DifferenceType::Add,
            Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_ADD),
            ansi_term::Style::new().fg(hwatch_diffmode::COLOR_BATCH_LINE_ADD),
        ),
        ChangeTag::Equal => return gen_equal_row_from_line(line, line_index, options),
    };

    let line_data = ansi::get_ansi_strip_str(line);
    let line_data = line_data.trim_end_matches('\n').to_string();

    let watch_line = Line::from(vec![
        Span::styled(line_header.to_string(), tui_line_style),
        Span::styled(line_data.clone(), tui_line_style),
    ]);
    let batch_line = str_line_style.paint(line_data.clone()).to_string();

    Some(DiffRow {
        watch_line,
        batch_line,
        line_number: Some(line_index + 1),
        diff_type,
    })
}

fn gen_line_diff_row<'a>(
    change: &InlineChange<[u8]>,
    is_word_highlight: bool,
    options: &DiffModeOptions,
) -> Option<DiffRow<'a>> {
    let mut result_line_spans = vec![];
    let mut result_str_elements = vec![];

    // set variables related to output
    let line_number: i32;
    let line_header: &str;
    let diff_type: DifferenceType;
    let tui_line_style: Style;
    let tui_line_highlight_style: Style;
    let tui_line_header_style: Style;
    let str_line_style: ansi_term::Style;
    let str_line_highlight_style: ansi_term::Style;
    let strip_ansi: bool;
    match change.tag() {
        ChangeTag::Equal => {
            // If is_only_diffline is valid, it will not be output in the first place, so it will return here.
            if options.get_only_diffline() {
                return None;
            }

            line_number = change.old_index().unwrap() as i32;
            line_header = "   ";
            diff_type = DifferenceType::Same;
            tui_line_style = Style::default();
            tui_line_highlight_style = Style::default();
            tui_line_header_style =
                Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_NUMBER_DEFAULT);
            str_line_style = ansi_term::Style::new();
            str_line_highlight_style = ansi_term::Style::new();
            strip_ansi = false;
        }
        ChangeTag::Delete => {
            line_number = change.old_index().unwrap() as i32;
            line_header = "-  ";
            diff_type = DifferenceType::Rem;
            tui_line_style = Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_REM);
            tui_line_highlight_style = Style::default()
                .fg(hwatch_diffmode::COLOR_WATCH_LINE_REM)
                .reversed()
                .bg(hwatch_diffmode::COLOR_WATCH_LINE_REVERSE_FG);
            tui_line_header_style =
                Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_NUMBER_REM);
            str_line_style = ansi_term::Style::new().fg(hwatch_diffmode::COLOR_BATCH_LINE_REM);
            str_line_highlight_style = ansi_term::Style::new()
                .fg(hwatch_diffmode::COLOR_BATCH_LINE_REVERSE_FG)
                .on(hwatch_diffmode::COLOR_BATCH_LINE_REM);
            strip_ansi = true;
        }
        ChangeTag::Insert => {
            line_number = change.new_index().unwrap() as i32;
            line_header = "+  ";
            diff_type = DifferenceType::Add;
            tui_line_style = Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_ADD);
            tui_line_highlight_style = Style::default()
                .fg(hwatch_diffmode::COLOR_WATCH_LINE_ADD)
                .reversed()
                .bg(hwatch_diffmode::COLOR_WATCH_LINE_REVERSE_FG);
            tui_line_header_style =
                Style::default().fg(hwatch_diffmode::COLOR_WATCH_LINE_NUMBER_ADD);
            str_line_style = ansi_term::Style::new().fg(hwatch_diffmode::COLOR_BATCH_LINE_ADD);
            str_line_highlight_style = ansi_term::Style::new()
                .fg(hwatch_diffmode::COLOR_BATCH_LINE_REVERSE_FG)
                .on(hwatch_diffmode::COLOR_BATCH_LINE_ADD);
            strip_ansi = true;
        }
    };

    // create result_line and result_str
    result_line_spans.push(Span::styled(format!("{line_header}"), tui_line_style));
    result_str_elements.push(
        str_line_style
            .paint(format!("{line_header}").to_string())
            .to_string(),
    );
    for (emphasized, value) in change.iter_strings_lossy() {
        let mut line_data = value.to_string();
        if strip_ansi {
            line_data = ansi::get_ansi_strip_str(&line_data);
        }

        if is_word_highlight && emphasized {
            // word highlight
            // line push
            result_line_spans.push(Span::styled(
                format!("{line_data}"),
                tui_line_highlight_style,
            ));

            // str push
            result_str_elements.push(
                str_line_highlight_style
                    .paint(format!("{line_data}"))
                    .to_string(),
            );
        } else {
            // normal
            match change.tag() {
                ChangeTag::Equal => {
                    if options.get_color() {
                        result_line_spans = vec![Span::from(line_header)];
                        let colored_data = ansi::bytes_to_text(format!("{line_data}").as_bytes());

                        for d in colored_data.lines {
                            for x in d.spans {
                                result_line_spans.push(x);
                            }
                        }
                        result_str_elements.push(
                            str_line_style
                                .paint(format!("{line_data}").to_string())
                                .to_string(),
                        );
                    } else {
                        let color_strip_data = ansi::get_ansi_strip_str(&line_data);
                        result_line_spans
                            .push(Span::styled(format!("{line_data}"), tui_line_style));
                        result_str_elements.push(
                            str_line_style
                                .paint(format!("{color_strip_data}").to_string())
                                .to_string(),
                        );
                    }
                }
                _ => {
                    line_data = ansi::get_ansi_strip_str(&value);
                    let color_strip_data = ansi::get_ansi_strip_str(&line_data)
                        .trim_end_matches('\n')
                        .to_string();
                    result_line_spans.push(Span::styled(format!("{line_data}"), tui_line_style));
                    result_str_elements.push(
                        str_line_style
                            .paint(format!("{color_strip_data}").to_string())
                            .to_string(),
                    );
                }
            }
        }
    }

    let result_line = Line::from(result_line_spans);
    let result_str = result_str_elements
        .join("")
        .trim_end_matches('\n')
        .to_string();

    let _ = tui_line_header_style;
    Some(DiffRow {
        watch_line: result_line,
        batch_line: result_str.trim_end_matches('\n').to_string(),
        line_number: Some((line_number + 1) as usize),
        diff_type,
    })
}
