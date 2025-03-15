// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    prelude::Line,
    style::{Color, Style, Stylize},
    text::Span,
};

use hwatch_ansi as ansi;
use hwatch_diffmode::{
    expand_output_vec_element_data, gen_counter_str, DiffMode, DiffModeExt, DiffModeOptions,
    DifferenceType, OutputVecData, OutputVecElementData,
};
use similar::{ChangeTag, InlineChange, TextDiff};
use std::cmp;

pub struct DiffModeAtLineDiff {
    header_width: usize,
    options: DiffModeOptions,
}

impl DiffModeAtLineDiff {
    pub fn new() -> Self {
        Self {
            header_width: 3,
            options: DiffModeOptions::new(),
        }
    }
}

impl DiffMode for DiffModeAtLineDiff {
    fn generate_watch_diff(&mut self, dest: &str, src: &str) -> Vec<Line<'static>> {
        // create LineDiffOptions
        let options: LineDiffOptions = LineDiffOptions {
            is_color: self.options.get_color(),
            is_batch: false,
            is_line_number: self.options.get_line_number(),
            is_word_highlight: self.options.get_word_highlight(),
            is_only_diffline: self.options.get_only_diffline(),
        };

        // generate diff output
        let (header_width, output_vec_data) = gen_line_diff_output(dest, src, &options);
        self.header_width = header_width;

        if let OutputVecData::Lines(lines) = output_vec_data {
            return lines;
        } else {
            return vec![];
        }
    }

    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String> {
        // create LineDiffOptions
        let options: LineDiffOptions = LineDiffOptions {
            is_color: self.options.get_color(),
            is_batch: true,
            is_line_number: self.options.get_line_number(),
            is_word_highlight: self.options.get_word_highlight(),
            is_only_diffline: self.options.get_only_diffline(),
        };

        // generate diff output
        let (header_width, output_vec_data) = gen_line_diff_output(dest, src, &options);
        self.header_width = header_width;

        if let OutputVecData::Strings(lines) = output_vec_data {
            return lines;
        } else {
            return vec![];
        }
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
        self.header_width
    }
}

// ----
// private function
// ----

struct LineDiffOptions {
    is_color: bool,
    is_batch: bool,
    is_line_number: bool,
    is_word_highlight: bool,
    is_only_diffline: bool,
}

// TODO: options系はstructで渡すようにする
fn gen_line_diff_output<'a>(
    dest: &str,
    src: &str,
    options: &LineDiffOptions,
) -> (usize, OutputVecData<'a>) {
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
    let mut result_line = vec![];
    let mut result_str = vec![];
    for op in diff_set.ops().iter() {
        for change in diff_set.iter_inline_changes(op) {
            // create PrintElementData
            let data = gen_line_diff_element(&change, header_width, options);
            match data {
                OutputVecElementData::String(data_str) => result_str.push(data_str),
                OutputVecElementData::Line(data_line) => result_line.push(data_line),
                OutputVecElementData::None() => {}
            }
        }
    }

    if options.is_batch {
        return (header_width, OutputVecData::Strings(result_str));
    } else {
        return (header_width, OutputVecData::Lines(result_line));
    }
}

//
fn gen_line_diff_element<'a>(
    change: &InlineChange<[u8]>,
    header_width: usize,
    options: &LineDiffOptions,
) -> OutputVecElementData<'a> {
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
    match change.tag() {
        ChangeTag::Equal => {
            // If is_only_diffline is valid, it will not be output in the first place, so it will return here.
            if options.is_only_diffline {
                return OutputVecElementData::None();
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
        if options.is_word_highlight && emphasized {
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
                    if options.is_color {
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

    let mut result_line = Line::from(result_line_spans);
    let mut result_str = result_str_elements
        .join("")
        .trim_end_matches('\n')
        .to_string();

    // add line number
    if options.is_line_number {
        let line_number = line_number + 1;
        // result_line update
        result_line.spans.insert(
            0,
            Span::styled(
                format!("{line_number:>header_width$} | "),
                tui_line_header_style,
            ),
        );

        result_str.insert_str(
            0,
            &gen_counter_str(
                options.is_color,
                line_number as usize,
                header_width,
                diff_type,
            ),
        );
    }

    if options.is_batch {
        return OutputVecElementData::String(result_str.trim_end_matches('\n').to_string());
    } else {
        return OutputVecElementData::Line(result_line);
    }
}
