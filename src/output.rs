// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// modules
use ansi_parser::{AnsiParser, AnsiSequence, Output};
use difference::{Changeset, Difference};
use heapless::consts::*;
use regex::Regex;
use std::{borrow::Cow, vec};
use std::cmp;
use std::fmt::Write;
use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    prelude::Line,
};
use ansi_term::Colour;

// local const
use crate::ansi;
use crate::LINE_ENDING;
use crate::DEFAULT_TAB_SIZE;

// local module
use crate::common::{DiffMode, OutputMode};
use crate::exec::CommandResult;

// output return type
enum PrintData<'a> {
    Lines(Vec<Line<'a>>),
    Strings(Vec<String>),
}

enum PrintElementData<'a> {
    Line(Line<'a>),
    String(String),
}

enum DifferenceType {
    Same,
    Add,
    Rem,
}

pub trait StringExt {
    fn expand_tabs(&self, tab_size: u16) -> Cow<str>;
}

impl<T> StringExt for T
where
    T: AsRef<str>,
{
    fn expand_tabs(&self, tab_size: u16) -> Cow<str> {
        let s = self.as_ref();
        let tab = '\t';

        if s.contains(tab) {
            let mut res = String::new();
            let mut last_pos = 0;

            while let Some(pos) = &s[last_pos..].find(tab) {
                res.push_str(&s[last_pos..*pos + last_pos]);

                let spaces_to_add = if tab_size != 0 {
                    tab_size - (*pos as u16 % tab_size)
                } else {
                    0
                };

                if spaces_to_add != 0 {
                    let _ = write!(res, "{:width$}", "", width = spaces_to_add as usize);
                }

                last_pos += *pos + 1;
            }

            res.push_str(&s[last_pos..]);

            Cow::from(res)
        } else {
            Cow::from(s)
        }
    }
}

pub struct Printer {
    // diff mode.
    diff_mode: DiffMode,

    // output mode.
    output_mode: OutputMode,

    // batch mode.
    is_batch: bool,

    // color mode.
    is_color: bool,

    // line number.
    is_line_number: bool,

    // is filtered text.
    is_filter: bool,

    // is regex filter text.
    is_regex_filter: bool,

    // filter text.
    filter_text: String,

    // is only print different line.
    is_only_diffline: bool,

    // tab size.
    tab_size: u16,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            diff_mode: DiffMode::Disable,
            output_mode: OutputMode::Output,
            is_batch: false,
            is_color: false,
            is_line_number: false,
            is_filter: false,
            is_regex_filter: false,
            filter_text: "".to_string(),
            is_only_diffline: false,
            tab_size: DEFAULT_TAB_SIZE,
        }
    }

    ///
    pub fn get_watch_text<'a>(&mut self, dest: CommandResult, src: CommandResult) -> Vec<Line<'a>> {
        // set new text(text_dst)
        let text_dest = match self.output_mode {
            OutputMode::Output => dest.output,
            OutputMode::Stdout => dest.stdout,
            OutputMode::Stderr => dest.stderr,
        };

        // set old text(text_src)
        let text_src = match self.output_mode {
            OutputMode::Output => src.output,
            OutputMode::Stdout => src.stdout,
            OutputMode::Stderr => src.stderr,
        };

        let data = match self.diff_mode {
            DiffMode::Disable => self.gen_plane_output(&text_dest),
            DiffMode::Watch => self.gen_watch_diff_output(&text_dest, &text_src),
            DiffMode::Line => self.gen_line_diff_output(&text_dest, &text_src),
            DiffMode::Word => self.gen_plane_output(&text_dest),
        };

        if let PrintData::Lines(result) = data {
            return result;
        } else {
            return vec![];
        }
    }

    ///
    pub fn get_batch_text(&mut self, dest: CommandResult, src: CommandResult) -> Vec<String> {
        // set new text(text_dst)
        let mut text_dest = match self.output_mode {
            OutputMode::Output => dest.output,
            OutputMode::Stdout => dest.stdout,
            OutputMode::Stderr => dest.stderr,
        };

        // set old text(text_src)
        let mut text_src = match self.output_mode {
            OutputMode::Output => src.output,
            OutputMode::Stdout => src.stdout,
            OutputMode::Stderr => src.stderr,
        };

        if !self.is_color {
            text_dest = get_ansi_strip_str(&text_dest);
            text_src = get_ansi_strip_str(&text_src);
        }

        let data = match self.diff_mode {
            DiffMode::Disable => self.gen_plane_output(&text_dest),
            DiffMode::Watch => self.gen_watch_diff_output(&text_dest, &text_src),
            DiffMode::Line => self.gen_line_diff_output(&text_dest, &text_src),
            DiffMode::Word => self.gen_plane_output(&text_dest),
        };

        if let PrintData::Strings(result) = data {
            return result;
        } else {
            return vec![];
        }
    }

    // Plane Output
    // ====================
    /// generate output at DiffMOde::Disable
    fn gen_plane_output<'a>(&mut self, dest: &str) -> PrintData<'a> {
        // tab expand
        let mut text = dest.to_string();
        if !self.is_batch {
            text = expand_line_tab(dest, self.tab_size);
        }

        if self.is_batch {
            if self.is_filter && !self.is_color {
                return self.create_plane_output_batch_with_filter(&text);
            } else {
                return self.create_plane_output_batch(&text);
            }
        } else {
            if self.is_filter && !self.is_color {
                return self.create_plane_output_watch_with_filter(&text);
            } else {
                return self.create_plane_output_watch(&text);
            }
        }
    }

    /// create filter pattern
    fn create_filter_pattern(&mut self) -> Regex {
        // set filter regex.
        let mut pattern: Regex = Regex::new(&self.filter_text).unwrap();
        if !self.is_color && self.is_filter {
            pattern = Regex::new(&regex::escape(&self.filter_text)).unwrap();

            if self.is_regex_filter {
                pattern = Regex::new(&self.filter_text).unwrap();
            }
        }

        return pattern;
    }

    ///
    fn create_plane_output_batch<'a>(&mut self, text: &str) -> PrintData<'a> {
        //
        let mut result = Vec::new();

        //
        let header_width = text.split('\n').count().to_string().chars().count();
        let mut counter = 1;

        // split line
        for l in text.split('\n') {
            let mut line = String::new();
            if self.is_line_number {
                line.push_str(
                    &gen_counter_str(self.is_color, counter, header_width, DifferenceType::Same)
                );
            }

            line.push_str(&l);
            result.push(line);

            counter += 1;
        }

        return PrintData::Strings(result);
    }

    ///
    fn create_plane_output_batch_with_filter<'a>(&mut self, text: &str) -> PrintData<'a> {
        // @TODO: filterを適用するオプションをそのうち作る
        //
        let mut result = Vec::new();

        //
        let header_width = text.split('\n').count().to_string().chars().count();
        let mut counter = 1;

        // split line
        for l in text.split('\n') {
            let mut line = String::new();
            if self.is_line_number {
                line.push_str(&gen_counter_str(self.is_color, counter, header_width, DifferenceType::Same));
            }

            line.push_str(&l);
            result.push(line);

            counter += 1;
        }

        return PrintData::Strings(result);
    }

    ///
    fn create_plane_output_watch<'a>(&mut self, text: &str) -> PrintData<'a> {
        //
        let mut result = Vec::new();

        //
        let header_width = text.split('\n').count().to_string().chars().count();
        let mut counter = 1;

        // split line
        for l in text.split('\n') {
            let mut line = vec![];

            if self.is_line_number {
                line.push(Span::styled(
                    format!("{counter:>header_width$} | "),
                Style::default().fg(Color::DarkGray),
                ));
            }

            if self.is_color {
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

        return PrintData::Lines(result);
    }

    ///
    fn create_plane_output_watch_with_filter<'a>(&mut self, text: &str) -> PrintData<'a> {
        let mut result = Vec::new();

        // line_span is vec for span on a line-by-line basis
        let mut line_span = vec![];

        let pattern = self.create_filter_pattern();

        // line number
        let mut counter = 1;
        let header_width = &text.split('\n').clone().count().to_string().chars().count();

        let mut last_match: usize = 0;

        for mch in pattern.find_iter(text) {
            let start: usize = mch.start();
            let end: usize = mch.end();

            // before regex hit.
            let before_range_text = &text[last_match..start];

            // regex hit.
            let range_text = &text[start..end];

            // split newline to Spans, at before_range_text
            for (before_range_count, before_text_line) in before_range_text.split('\n').enumerate()
            {
                if before_range_count > 0 {
                    let line_data = line_span.clone();
                    result.push(Line::from(line_data));
                    line_span = vec![];
                    counter += 1;
                }

                if self.is_line_number && line_span.is_empty() {
                    line_span.push(Span::styled(
                        format!("{counter:>header_width$} | "),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                // push to line_span at before_text_line.
                line_span.push(Span::from(before_text_line.to_string()));
            }

            // split newline to Spans, at range_text
            for (range_count, text_line) in range_text.split('\n').enumerate() {
                if range_count > 0 {
                    if self.is_line_number && line_span.is_empty() {
                        line_span.push(Span::styled(
                            format!("{counter:>header_width$} | "),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    let line_data = line_span.clone();
                    result.push(Line::from(line_data));
                    line_span = vec![];
                    counter += 1;
                }

                // push to line_span at text_line.
                line_span.push(Span::styled(
                    text_line.to_string(),
                    Style::default().add_modifier(Modifier::REVERSED),
                ));
            }

            last_match = end;
        }

        // last push
        let last_str = &text[last_match..];

        for last_str_line in last_str.split('\n') {
            let mut last_str_line_span = vec![];
            if !line_span.is_empty() {
                last_str_line_span = line_span;
                line_span = vec![];
            }

            if self.is_line_number && last_str_line_span.is_empty() {
                last_str_line_span.push(Span::styled(
                    format!("{counter:>header_width$} | "),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            last_str_line_span.push(Span::from(String::from(last_str_line)));

            result.push(Line::from(last_str_line_span));

            counter += 1;
        }

        return PrintData::Lines(result);
    }

    // Watch Diff Output
    // ====================
    /// generate output at DiffMOde::Watch
    fn gen_watch_diff_output<'a>(&mut self, dest: &str, src: &str) -> PrintData<'a> {
        // tab expand dest
        let mut text_dest = dest.to_string();
        if !self.is_batch {
            text_dest = expand_line_tab(dest, self.tab_size);
        }

        // tab expand src
        let mut text_src = src.to_string();
        if !self.is_batch {
            text_src = expand_line_tab(src, self.tab_size);
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

            let mut line_data = match self.is_color {
                false => self.create_watch_diff_output_line(&src_line, &dest_line),
                true => {
                    if self.is_batch {
                        self.create_watch_diff_output_line(&src_line, &dest_line)
                    } else {
                        self.create_watch_diff_output_line_with_ansi_for_watch(&src_line, &dest_line)
                    }
                },
            };

            if self.is_line_number {
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
                            &gen_counter_str(self.is_color, counter, header_width, DifferenceType::Same)
                        );
                    }
                }
            };

            result.push(line_data);
            counter += 1;
        }

        return expand_print_element_data(self.is_batch, result);


    }

    ///
    fn create_watch_diff_output_line<'a>(&mut self, src_line: &str, dest_line: &str) -> PrintElementData<'a> {
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
            return PrintElementData::Line(Line::from(result_spans))
        }
    }

    ///
    fn create_watch_diff_output_line_with_ansi_for_watch<'a>(&mut self, src_line: &str, dest_line: &str) -> PrintElementData<'a> {
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
            if src_spans[x].content != dest_spans[x].content || src_spans[x].style != dest_spans[x].style
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

    // Line Diff Output
    // ====================
    ///
    fn gen_line_diff_output<'a>(&mut self, dest: &str, src: &str) -> PrintData<'a> {
        // tab expand dest
        let mut text_dest = dest.to_string();
        if !self.is_batch {
            text_dest = expand_line_tab(dest, self.tab_size);
        }

        // tab expand src
        let mut text_src = src.to_string();
        if !self.is_batch {
            text_src = expand_line_tab(src, self.tab_size);
        }

        // Create changeset
        let Changeset { diffs, .. } = Changeset::new(&text_src, &text_dest, LINE_ENDING);

        // src and dest text's line count.
        let src_len = &text_src.lines().count();
        let dest_len = &text_dest.lines().count();

        // get line_number width
        let header_width = cmp::max(src_len, dest_len).to_string().chars().count();

        // line_number counter
        let mut src_counter = 1;
        let mut dest_counter = 1;

        // create result
        let mut result_line = vec![];
        let mut result_str = vec![];

        (0..diffs.len()).for_each(|i| {
            match diffs[i] {
                // Same line.
                Difference::Same(ref diff_data) => {
                    for l in diff_data.lines() {
                        let data = self.gen_line_diff_linedata_from_diffs_str(
                            l,
                            DifferenceType::Same,
                            dest_counter,
                            header_width,
                        );

                        if !self.is_only_diffline {
                            match data {
                                PrintElementData::String(data_str) => result_str.push(data_str),
                                PrintElementData::Line(data_line) => result_line.push(data_line),
                            }
                        }

                        // add counter
                        src_counter += 1;
                        dest_counter += 1;
                    }
                }

                // Add line.
                Difference::Add(ref diff_data) => {
                    for l in diff_data.lines() {
                        let data = self.gen_line_diff_linedata_from_diffs_str(
                            l,
                            DifferenceType::Add,
                            dest_counter,
                            header_width,
                        );

                        match data {
                            PrintElementData::String(data_str) => result_str.push(data_str),
                            PrintElementData::Line(data_line) => result_line.push(data_line),
                        }

                        // add counter
                        dest_counter += 1;
                    }
                }

                // Remove line.
                Difference::Rem(ref diff_data) => {
                    for l in diff_data.lines() {
                        let data = self.gen_line_diff_linedata_from_diffs_str(
                            l,
                            DifferenceType::Rem,
                            src_counter,
                            header_width,
                        );

                        match data {
                            PrintElementData::String(data_str) => result_str.push(data_str.trim_end().to_string()),
                            PrintElementData::Line(data_line) => result_line.push(data_line),
                        }

                        // add counter
                        src_counter += 1;
                    }
                }
            }
        });

        if self.is_batch {
            return PrintData::Strings(result_str);
        } else {
            return PrintData::Lines(result_line);
        }
    }

    ///
    fn gen_line_diff_linedata_from_diffs_str<'a>(
        &mut self,
        diff_line: &str,
        diff_type: DifferenceType,
        line_number: i32,
        header_width: usize,
    ) -> PrintElementData<'a> {
        //
        let line_header: &str;
        let tui_line_style: Style;
        let tui_line_header_style: Style;
        let str_line_style: ansi_term::Style;
        // let str_line_header_style: ansi_term::Style;

        match diff_type {
            DifferenceType::Same => {
                line_header = "   ";
                tui_line_style = Style::default();
                tui_line_header_style = Style::default().fg(Color::DarkGray);
                str_line_style = ansi_term::Style::new();
                // str_line_header_style = ansi_term::Style::new().fg(Colour::Fixed(240));
            },

            DifferenceType::Add => {
                line_header = "+  ";
                tui_line_style = Style::default().fg(Color::Green);
                tui_line_header_style = Style::default().fg(Color::Rgb(56, 119, 120));
                str_line_style = ansi_term::Style::new().fg(Colour::Green);
                // str_line_header_style = ansi_term::Style::new().fg(Colour::RGB(56, 119, 120));
            },

            DifferenceType::Rem => {
                line_header = "-  ";
                tui_line_style = Style::default().fg(Color::Red);
                tui_line_header_style = Style::default().fg(Color::Rgb(118, 0, 0));
                str_line_style = ansi_term::Style::new().fg(Colour::Red);
                // str_line_header_style = ansi_term::Style::new().fg(Colour::RGB(118, 0, 0));
            },
        };

        // create result_line
        let mut result_line =  match diff_type {
            DifferenceType::Same => {
                if self.is_color {
                    let mut colored_span = vec![Span::from(line_header)];
                    let colored_data = ansi::bytes_to_text(format!("{diff_line}\n").as_bytes());
                    for d in colored_data.lines {
                        for x in d.spans {
                            colored_span.push(x);
                        }
                    }
                    Line::from(colored_span)
                } else {
                    Line::from(format!("{line_header}{diff_line}\n"))
                }

            },

            _ => {
                let mut line_data = diff_line.to_string();
                if self.is_color {
                    line_data = get_ansi_strip_str(&diff_line);
                }

                Line::from(
                    Span::styled(format!("{line_header}{line_data}\n"), tui_line_style)
                )
            },
        };

        // create result_str
        let mut result_str = match diff_type {
            DifferenceType::Same => {
                let mut line_data = format!("{line_header}{diff_line}");
                if !self.is_color {
                    line_data = get_ansi_strip_str(&line_data);
                }
                line_data
            },

            _ => {
                let mut line_data = format!("{line_header}{diff_line}");
                if self.is_color {
                    line_data = str_line_style.paint(
                        get_ansi_strip_str(&format!("{line_header}{diff_line}"))
                    ).to_string();
                }
                line_data
            },
        };

        // add line number
        if self.is_line_number {
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
                &gen_counter_str(self.is_color, line_number as usize, header_width, diff_type)
            );
        }


        if self.is_batch {
            return PrintElementData::String(result_str.to_string().trim_end().to_string());
        } else {
            return PrintElementData::Line(result_line);
        }
    }

    // Word Diff Output
    // ====================
    ///
    ///















    /// set diff mode.
    pub fn set_diff_mode(&mut self, diff_mode: DiffMode) -> &mut Self {
        self.diff_mode = diff_mode;
        self
    }

    /// set output mode.
    pub fn set_output_mode(&mut self, output_mode: OutputMode) -> &mut Self {
        self.output_mode = output_mode;
        self
    }

    /// set batch mode.
    pub fn set_batch(&mut self, is_batch: bool) -> &mut Self {
        self.is_batch = is_batch;
        self
    }

    /// set color mode.
    pub fn set_color(&mut self, is_color: bool) -> &mut Self {
        self.is_color = is_color;
        self
    }

    /// set line number.
    pub fn set_line_number(&mut self, is_line_number: bool) -> &mut Self {
        self.is_line_number = is_line_number;
        self
    }

    /// set is_filter.
    pub fn set_filter(&mut self, is_filter: bool) -> &mut Self {
        self.is_filter = is_filter;
        self
    }

    /// set diff mode.
    pub fn set_regex_filter(&mut self, is_regex_filter: bool) -> &mut Self {
        self.is_regex_filter = is_regex_filter;
        self
    }

    /// set filter text.
    pub fn set_filter_text(&mut self, filter_text: String) -> &mut Self {
        self.filter_text = filter_text;
        self
    }

    /// set diff mode.
    pub fn set_only_diffline(&mut self, is_only_diffline: bool) -> &mut Self {
        self.is_only_diffline = is_only_diffline;
        self
    }

    /// set tab size.
    pub fn set_tab_size(&mut self, tab_size: u16) -> &mut Self {
        self.tab_size = tab_size;
        self
    }
}

///
fn expand_line_tab(data: &str, tab_size: u16) -> String {
    let mut result_vec: Vec<String> = vec![];
    for d in data.lines() {
        let l = d.expand_tabs(tab_size).to_string();
        result_vec.push(l);
    }

    result_vec.join("\n")
}

///
fn gen_counter_str(is_color: bool,counter: usize, header_width: usize, diff_type: DifferenceType) -> String {
    let mut counter_str = counter.to_string();
    let mut seprator = " | ".to_string();
    let mut prefix_width = 0;
    let mut suffix_width = 0;

    if is_color {
        let style: ansi_term::Style = match diff_type {
            DifferenceType::Same => ansi_term::Style::default().fg(Colour::Fixed(240)),
            DifferenceType::Add => ansi_term::Style::default().fg(Colour::RGB(56, 119, 120)),
            DifferenceType::Rem => ansi_term::Style::default().fg(Colour::RGB(118, 0, 0)),
        };
        counter_str = style.paint(counter_str).to_string();
        seprator = style.paint(seprator).to_string();
        prefix_width = style.prefix().to_string().len();
        suffix_width = style.suffix().to_string().len();

    }

    let width = header_width + prefix_width + suffix_width;
    format!("{counter_str:>width$}{seprator}")
}

///
fn expand_print_element_data(is_batch: bool, data: Vec<PrintElementData>) -> PrintData {
    let mut lines = Vec::new();
    let mut strings = Vec::new();

    for element in data {
        match element {
            PrintElementData::Line(line) => {
                lines.push(line);
            }
            PrintElementData::String(string) => {
                strings.push(string);
            }
        }
    }

    if is_batch {
        return PrintData::Strings(strings)
    } else {
        return PrintData::Lines(lines)
    };
}

// word diff
// ==========

///
pub fn get_word_diff<'a>(
    color: bool,
    line_number: bool,
    is_only_diffline: bool,
    old: &str,
    new: &str,
    tab_size: u16,
) -> Vec<Line<'a>> {
    let old_text = &expand_line_tab(old, tab_size);
    let new_text = &expand_line_tab(new, tab_size);

    // Create changeset
    let Changeset { diffs, .. } = Changeset::new(&old_text, &new_text, LINE_ENDING);

    // old and new text's line count.
    let old_len = &old_text.lines().count();
    let new_len = &new_text.lines().count();

    // get line_number width
    let header_width = cmp::max(old_len, new_len).to_string().chars().count();

    // line_number counter
    let mut old_counter = 1;
    let mut new_counter = 1;

    // create result
    let mut result = vec![];

    for i in 0..diffs.len() {
        match diffs[i] {
            // Same line.
            Difference::Same(ref diff_data) => {
                for l in diff_data.lines() {
                    let line = l.expand_tabs(tab_size);
                    let mut data = if color {
                        // ansi color code => rs-tui colored span.
                        let mut colored_span = vec![Span::from("   ")];
                        let colored_data = ansi::bytes_to_text(format!("{line}\n").as_bytes());
                        for d in colored_data.lines {
                            for x in d.spans {
                                colored_span.push(x);
                            }
                        }
                        Line::from(colored_span)
                    } else {
                        // to string => rs-tui span.
                        Line::from(format!("   {line}\n"))
                    };

                    if line_number {
                        data.spans.insert(
                            0,
                            Span::styled(
                                format!("{new_counter:>header_width$} | "),
                                Style::default().fg(Color::DarkGray),
                            ),
                        );
                    }

                    if !is_only_diffline {
                        result.push(data);
                    }

                    // add counter
                    old_counter += 1;
                    new_counter += 1;
                }
            }

            // Add line.
            Difference::Add(ref diff_data) => {
                // line Spans.
                // it is lines data <Vec<Vec<Span<'a>>>>
                // ex)
                // [   // 1st line...
                //     [Sapn, Span, Span, ...],
                //     // 2nd line...
                //     [Sapn, Span, Span, ...],
                //     // 3rd line...
                //     [Sapn, Span, Span, ...],
                // ]
                let mut lines_data = vec![];

                // check lines.
                if i > 0 {
                    let before_diffs = &diffs[i - 1];

                    lines_data = get_word_diff_addline(color, before_diffs, diff_data.to_string())
                } else {
                    for l in diff_data.lines() {
                        let line = l.expand_tabs(tab_size);
                        let data = if color {
                            get_ansi_strip_str(&line)
                        } else {
                            line.to_string()
                        };
                        lines_data.push(vec![Span::styled(
                            data.to_string(),
                            Style::default().fg(Color::Green),
                        )]);
                    }
                }

                for line_data in lines_data {
                    let mut data = vec![Span::styled("+  ", Style::default().fg(Color::Green))];
                    for line in line_data {
                        data.push(line);
                    }

                    if line_number {
                        data.insert(
                            0,
                            Span::styled(
                                format!("{new_counter:>header_width$} | "),
                                Style::default().fg(Color::Rgb(56, 119, 120)),
                            ),
                        );
                    }

                    result.push(Line::from(data.clone()));

                    // add new_counter
                    new_counter += 1;
                }
            }

            // Remove line.
            Difference::Rem(ref diff_data) => {
                // line Spans.
                // it is lines data <Vec<Vec<Span<'a>>>>
                // ex)
                // [   // 1st line...
                //     [Sapn, Span, Span, ...],
                //     // 2nd line...
                //     [Sapn, Span, Span, ...],
                //     // 3rd line...
                //     [Sapn, Span, Span, ...],
                // ]
                let mut lines_data = vec![];

                // check lines.
                if diffs.len() > i + 1 {
                    let after_diffs: &Difference = &diffs[i + 1];

                    lines_data = get_word_diff_remline(color, after_diffs, diff_data.to_string())
                } else {
                    for line in diff_data.lines() {
                        let data = if color {
                            get_ansi_strip_str(line)
                        } else {
                            line.to_string()
                        };
                        lines_data.push(vec![Span::styled(
                            data.to_string(),
                            Style::default().fg(Color::Red),
                        )]);
                    }
                }

                for line_data in lines_data {
                    let mut data = vec![Span::styled("-  ", Style::default().fg(Color::Red))];
                    for line in line_data {
                        data.push(line);
                    }

                    if line_number {
                        data.insert(
                            0,
                            Span::styled(
                                format!("{old_counter:>header_width$} | "),
                                Style::default().fg(Color::Rgb(118, 0, 0)),
                            ),
                        );
                    }

                    result.push(Line::from(data.clone()));

                    // add old_counter
                    old_counter += 1;
                }
            }
        }
    }

    result
}

/// This Function when there is an additional line in word_diff and there is a previous diff.
///
fn get_word_diff_addline<'a>(
    color: bool,
    before_diffs: &difference::Difference,
    diff_data: String,
) -> Vec<Vec<Span<'a>>> {
    // result is Vec<Vec<Span>>
    // ex)
    // [   // 1st line...
    //     [Sapn, Span, Span, ...],
    //     // 2nd line...
    //     [Sapn, Span, Span, ...],
    //     // 3rd line...
    //     [Sapn, Span, Span, ...],
    // ]
    let mut result = vec![];

    // line_data is Vec<Span>
    // ex) [Span, Span, Span, ...]
    let mut line_data = vec![];

    match before_diffs {
        // Change Line.
        Difference::Rem(before_diff_data) => {
            // Craete Changeset at `Addlind` and `Before Diff Data`.
            let Changeset { diffs, .. } = Changeset::new(before_diff_data, &diff_data, " ");

            //
            for c in diffs {
                match c {
                    // Same
                    Difference::Same(ref char) => {
                        let same_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::Green),
                            char,
                        );

                        for (counter, lines) in same_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // Add
                    Difference::Add(ref char) => {
                        let add_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::White).bg(Color::Green),
                            char,
                        );

                        for (counter, lines) in add_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // No data.
                    _ => {}
                }
            }
        }

        // Add line
        _ => {
            for line in diff_data.lines() {
                let data = if color {
                    get_ansi_strip_str(line)
                } else {
                    line.to_string()
                };
                let line_data = vec![Span::styled(
                    data.to_string(),
                    Style::default().fg(Color::Green),
                )];
                result.push(line_data);
            }
        }
    }

    if !line_data.is_empty() {
        result.push(line_data);
    }

    result
}

///
fn get_word_diff_remline<'a>(
    color: bool,
    after_diffs: &difference::Difference,
    diff_data: String,
) -> Vec<Vec<Span<'a>>> {
    // result is Vec<Vec<Span>>
    // ex)
    // [   // 1st line...
    //     [Sapn, Span, Span, ...],
    //     // 2nd line...
    //     [Sapn, Span, Span, ...],
    //     // 3rd line...
    //     [Sapn, Span, Span, ...],
    // ]
    let mut result = vec![];

    // line_data is Vec<Span>
    // ex) [Span, Span, Span, ...]
    let mut line_data = vec![];

    match after_diffs {
        // Change Line.
        Difference::Add(after_diffs_data) => {
            // Craete Changeset at `Addlind` and `Before Diff Data`.
            let Changeset { diffs, .. } = Changeset::new(&diff_data, after_diffs_data, " ");

            //
            for c in diffs {
                match c {
                    // Same
                    Difference::Same(ref char) => {
                        let same_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::Red),
                            char,
                        );

                        for (counter, lines) in same_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // Add
                    Difference::Rem(ref char) => {
                        let add_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::White).bg(Color::Red),
                            char,
                        );

                        for (counter, lines) in add_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // No data.
                    _ => {}
                }
            }
        }

        // Rem line
        _ => {
            for line in diff_data.lines() {
                let data = if color {
                    get_ansi_strip_str(line)
                } else {
                    line.to_string()
                };

                let line_data = vec![Span::styled(
                    data.to_string(),
                    Style::default().fg(Color::Red),
                )];

                result.push(line_data);
            }
        }
    }

    if !line_data.is_empty() {
        result.push(line_data);
    }

    result
}

///
fn get_word_diff_line_to_spans<'a>(
    color: bool,
    style: Style,
    diff_str: &str,
) -> Vec<Vec<Span<'a>>> {
    // result
    let mut result = vec![];

    for l in diff_str.split('\n') {
        let text = if color {
            get_ansi_strip_str(l)
        } else {
            l.to_string()
        };

        let line = vec![
            Span::styled(text.clone(), style),
            Span::styled(" ", Style::default()),
        ];

        result.push(line);
    }

    result
}

// Ansi Color Code parse
// ==========

/// Apply ANSI color code character by character.
fn gen_ansi_all_set_str<'b>(text: &str) -> Vec<Vec<Span<'b>>> {
    // set Result
    let mut result = vec![];

    // ansi reset code heapless_vec
    let mut ansi_reset_vec = heapless::Vec::<u8, U5>::new();
    let _ = ansi_reset_vec.push(0);

    // get ansi reset code string
    let ansi_reset_seq = AnsiSequence::SetGraphicsMode(ansi_reset_vec);
    let ansi_reset_seq_str = ansi_reset_seq.to_string();

    // init sequence.
    let mut sequence: AnsiSequence;
    let mut sequence_str = "".to_string();

    // text processing
    let mut processed_text = vec![];
    for block in text.ansi_parse() {
        match block {
            Output::TextBlock(text) => {
                for char in text.chars() {
                    let append_text = if !sequence_str.is_empty() {
                        format!("{sequence_str}{char}{ansi_reset_seq_str}")
                    } else {
                        format!("{char}")
                    };

                    // parse ansi text to tui text.
                    let data = ansi::bytes_to_text(format!("{append_text}\n").as_bytes());
                    if let Some(d) = data.into_iter().next() {
                        for x in d.spans {
                            processed_text.push(x);
                        }
                    }
                }
            }
            Output::Escape(seq) => {
                sequence = seq;
                sequence_str = sequence.to_string();
            }
        }
    }

    result.push(processed_text);

    result
}

///
fn get_ansi_strip_str(text: &str) -> String {
    let mut line_str = "".to_string();
    for block in text.ansi_parse() {
        if let Output::TextBlock(t) = block {
            line_str.push_str(t);
        }
    }

    line_str
}
