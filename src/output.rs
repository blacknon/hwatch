// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: watch diffのハイライトについて、旧watchと同様のパターンとは別により見やすいハイライトを実装する
//       => Watch(Word)とかにするか？？
//       => すぐには出来なさそう？なので、0.3.15にて対応とする？
// TODO: 行・文字差分数の取得を行うための関数を作成する(ここじゃないかも？？)

// modules
use ansi_term::Colour;
use ratatui::style::Stylize;
use similar::{ChangeTag, InlineChange, TextDiff};
use std::cmp;
use std::fmt::Write;
use std::{borrow::Cow, vec};
use tui::{
    prelude::Line,
    style::{Color, Modifier, Style},
    text::Span,
};

// const
const COLOR_BATCH_LINE_NUMBER_DEFAULT: Colour = Colour::Fixed(240);
const COLOR_BATCH_LINE_NUMBER_ADD: Colour = Colour::RGB(56, 119, 120);
const COLOR_BATCH_LINE_NUMBER_REM: Colour = Colour::RGB(118, 0, 0);
const COLOR_BATCH_LINE_ADD: Colour = Colour::Green;
const COLOR_BATCH_LINE_REM: Colour = Colour::Red;
const COLOR_BATCH_LINE_REVERSE_FG: Colour = Colour::White;
const COLOR_WATCH_LINE_NUMBER_DEFAULT: Color = Color::DarkGray;
const COLOR_WATCH_LINE_NUMBER_ADD: Color = Color::Rgb(56, 119, 120);
const COLOR_WATCH_LINE_NUMBER_REM: Color = Color::Rgb(118, 0, 0);
const COLOR_WATCH_LINE_ADD: Color = Color::Green;
const COLOR_WATCH_LINE_REM: Color = Color::Red;
const COLOR_WATCH_LINE_REVERSE_FG: Color = Color::White;

// local const
use crate::ansi;
use crate::DEFAULT_TAB_SIZE;

// local module
use crate::ansi::gen_ansi_all_set_str;
use crate::ansi::get_ansi_strip_str;
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
    None(),
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

    // is reverse text.
    is_reverse: bool,

    // is word highlight at line diff.
    is_word_highlight: bool,

    // is only print different line.
    is_only_diffline: bool,

    // tab size.
    tab_size: u16,

    // watch window header width.
    header_width: usize,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            diff_mode: DiffMode::Disable,
            output_mode: OutputMode::Output,
            is_batch: false,
            is_color: false,
            is_line_number: false,
            is_reverse: false,
            is_word_highlight: false,
            is_only_diffline: false,
            tab_size: DEFAULT_TAB_SIZE,
            header_width: 0,
        }
    }

    ///
    pub fn get_watch_text<'a>(
        &mut self,
        dest: &CommandResult,
        src: &CommandResult,
    ) -> Vec<Line<'a>> {
        // set new text(text_dst)
        let text_dest = match self.output_mode {
            OutputMode::Output => (*dest).get_output(),
            OutputMode::Stdout => (*dest).get_stdout(),
            OutputMode::Stderr => (*dest).get_stderr(),
        };

        // set old text(text_src)
        let text_src = match self.output_mode {
            OutputMode::Output => (*src).get_output(),
            OutputMode::Stdout => (*src).get_stdout(),
            OutputMode::Stderr => (*src).get_stderr(),
        };

        let data = match self.diff_mode {
            DiffMode::Disable => self.gen_plane_output(&text_dest),
            DiffMode::Watch => self.gen_watch_diff_output(&text_dest, &text_src),
            DiffMode::Line => {
                self.is_word_highlight = false;
                self.gen_line_diff_output(&text_dest, &text_src)
            }
            DiffMode::Word => {
                self.is_word_highlight = true;
                self.gen_line_diff_output(&text_dest, &text_src)
            }
        };

        if let PrintData::Lines(mut result) = data {
            // if is_reverse enable, flip upside down to result.
            if self.is_reverse {
                result.reverse();
            }
            result
        } else {
            vec![]
        }
    }

    ///
    pub fn get_batch_text(&mut self, dest: &CommandResult, src: &CommandResult) -> Vec<String> {
        // set new text(text_dst)
        let mut text_dest = match self.output_mode {
            OutputMode::Output => (*dest).get_output(),
            OutputMode::Stdout => (*dest).get_stdout(),
            OutputMode::Stderr => (*dest).get_stderr(),
        };

        // set old text(text_src)
        let mut text_src = match self.output_mode {
            OutputMode::Output => (*src).get_output(),
            OutputMode::Stdout => (*src).get_stdout(),
            OutputMode::Stderr => (*src).get_stderr(),
        };

        if !self.is_color {
            text_dest = get_ansi_strip_str(&text_dest);
            text_src = get_ansi_strip_str(&text_src);
        }

        let data = match self.diff_mode {
            DiffMode::Disable => self.gen_plane_output(&text_dest),
            DiffMode::Watch => self.gen_watch_diff_output(&text_dest, &text_src),
            DiffMode::Line => {
                self.is_word_highlight = false;
                self.gen_line_diff_output(&text_dest, &text_src)
            }
            DiffMode::Word => {
                self.is_word_highlight = true;
                self.gen_line_diff_output(&text_dest, &text_src)
            }
        };

        if let PrintData::Strings(mut result) = data {
            // if is_reverse enable, flip upside down to result.
            if self.is_reverse {
                result.reverse();
            }
            result
        } else {
            vec![]
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

            if !self.is_color {
                text = ansi::escape_ansi(&text);
            }
        }

        if self.is_batch {
            self.create_plane_output_batch(&text)
        } else {
            self.create_plane_output_watch(&text)
        }
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
                line.push_str(&gen_counter_str(
                    self.is_color,
                    counter,
                    header_width,
                    DifferenceType::Same,
                ));
            }

            line.push_str(l);
            result.push(line);

            counter += 1;
        }

        PrintData::Strings(result)
    }

    ///
    fn create_plane_output_watch<'a>(&mut self, text: &str) -> PrintData<'a> {
        //
        let mut result = Vec::new();

        //
        let header_width = text.split('\n').count().to_string().chars().count();
        let mut counter = 1;

        // split line
        for mut l in text.split('\n') {
            if l.is_empty() {
                l = "\u{200B}";
            }

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

        PrintData::Lines(result)
    }

    // Watch Diff Output
    // ====================
    /// generate output at DiffMOde::Watch
    fn gen_watch_diff_output<'a>(&mut self, dest: &str, src: &str) -> PrintData<'a> {
        // tab expand dest
        let mut text_dest_str = dest.to_string();
        if !self.is_batch {
            text_dest_str = expand_line_tab(dest, self.tab_size);

            if !self.is_color {
                text_dest_str = ansi::escape_ansi(&text_dest_str);
            }
        }

        let mut text_dest: String = "".to_string();
        for mut l in text_dest_str.lines() {
            if l.is_empty() {
                l = "\u{200B}";
            }

            text_dest.push_str(l);
            text_dest.push('\n');
        }

        // tab expand src
        let mut text_src_str = src.to_string();
        if !self.is_batch {
            text_src_str = expand_line_tab(src, self.tab_size);

            if !self.is_color {
                text_src_str = ansi::escape_ansi(&text_src_str);
            }
        }

        let mut text_src: String = "".to_string();
        for mut l in text_src_str.lines() {
            if l.is_empty() {
                l = "\u{200B}";
            }

            text_src.push_str(l);
            text_src.push('\n');
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
                false => self.create_watch_diff_output_line(src_line, dest_line),
                true => {
                    if self.is_batch {
                        self.create_watch_diff_output_line(src_line, dest_line)
                    } else {
                        self.create_watch_diff_output_line_with_ansi_for_watch(src_line, dest_line)
                    }
                }
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

        expand_print_element_data(self.is_batch, result)
    }

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
            PrintElementData::String(data_str)
        } else {
            PrintElementData::Line(Line::from(result_spans))
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

        PrintElementData::Line(Line::from(result))
    }

    ///
    fn gen_line_diff_output<'a>(&mut self, dest: &str, src: &str) -> PrintData<'a> {
        // tab expand dest
        let mut text_dest = dest.to_string();
        if !self.is_batch {
            text_dest = expand_line_tab(dest, self.tab_size);

            if !self.is_color {
                text_dest = ansi::escape_ansi(&text_dest);
            }
        }
        let text_dest_bytes = text_dest.as_bytes().to_vec();

        // tab expand src
        let mut text_src = src.to_string();
        if !self.is_batch {
            text_src = expand_line_tab(src, self.tab_size);

            if !self.is_color {
                text_src = ansi::escape_ansi(&text_src);
            }
        }
        let text_src_bytes = text_src.as_bytes().to_vec();

        // Create diff data
        let diff_set = TextDiff::from_lines(&text_src_bytes, &text_dest_bytes);

        // src and dest text's line count.
        let src_len = diff_set.old_slices().len();
        let dest_len = diff_set.new_slices().len();

        // get line_number width
        self.header_width = cmp::max(src_len, dest_len).to_string().chars().count();

        // create result
        let mut result_line = vec![];
        let mut result_str = vec![];
        for op in diff_set.ops().iter() {
            for change in diff_set.iter_inline_changes(op) {
                // create PrintElementData
                let data = self.gen_line_diff_element(&change);
                match data {
                    PrintElementData::String(data_str) => result_str.push(data_str),
                    PrintElementData::Line(data_line) => result_line.push(data_line),
                    PrintElementData::None() => {}
                }
            }
        }

        if self.is_batch {
            PrintData::Strings(result_str)
        } else {
            PrintData::Lines(result_line)
        }
    }

    //
    fn gen_line_diff_element<'a>(&mut self, change: &InlineChange<[u8]>) -> PrintElementData<'a> {
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
                if self.is_only_diffline {
                    return PrintElementData::None();
                }

                line_number = change.old_index().unwrap() as i32;
                line_header = "   ";
                diff_type = DifferenceType::Same;
                tui_line_style = Style::default();
                tui_line_highlight_style = Style::default();
                tui_line_header_style = Style::default().fg(COLOR_WATCH_LINE_NUMBER_DEFAULT);
                str_line_style = ansi_term::Style::new();
                str_line_highlight_style = ansi_term::Style::new();
            }
            ChangeTag::Delete => {
                line_number = change.old_index().unwrap() as i32;
                line_header = "-  ";
                diff_type = DifferenceType::Rem;
                tui_line_style = Style::default().fg(COLOR_WATCH_LINE_REM);
                tui_line_highlight_style = Style::default()
                    .fg(COLOR_WATCH_LINE_REM)
                    .reversed()
                    .bg(COLOR_WATCH_LINE_REVERSE_FG);
                tui_line_header_style = Style::default().fg(COLOR_WATCH_LINE_NUMBER_REM);
                str_line_style = ansi_term::Style::new().fg(COLOR_BATCH_LINE_REM);
                str_line_highlight_style = ansi_term::Style::new()
                    .fg(COLOR_BATCH_LINE_REVERSE_FG)
                    .on(COLOR_BATCH_LINE_REM);
            }
            ChangeTag::Insert => {
                line_number = change.new_index().unwrap() as i32;
                line_header = "+  ";
                diff_type = DifferenceType::Add;
                tui_line_style = Style::default().fg(COLOR_WATCH_LINE_ADD);
                tui_line_highlight_style = Style::default()
                    .fg(COLOR_WATCH_LINE_ADD)
                    .reversed()
                    .bg(COLOR_WATCH_LINE_REVERSE_FG);
                tui_line_header_style = Style::default().fg(COLOR_WATCH_LINE_NUMBER_ADD);
                str_line_style = ansi_term::Style::new().fg(COLOR_BATCH_LINE_ADD);
                str_line_highlight_style = ansi_term::Style::new()
                    .fg(COLOR_BATCH_LINE_REVERSE_FG)
                    .on(COLOR_BATCH_LINE_ADD);
            }
        };

        // create result_line and result_str
        result_line_spans.push(Span::styled(line_header.to_string(), tui_line_style));
        result_str_elements.push(
            str_line_style
                .paint(line_header.to_string().to_string())
                .to_string(),
        );
        for (emphasized, value) in change.iter_strings_lossy() {
            let mut line_data = value.to_string();
            if self.is_word_highlight && emphasized {
                // word highlight
                // line push
                result_line_spans.push(Span::styled(
                    line_data.to_string(),
                    tui_line_highlight_style,
                ));

                // str push
                result_str_elements.push(
                    str_line_highlight_style
                        .paint(line_data.to_string())
                        .to_string(),
                );
            } else {
                // normal
                match change.tag() {
                    ChangeTag::Equal => {
                        if self.is_color {
                            result_line_spans = vec![Span::from(line_header)];
                            let colored_data =
                                ansi::bytes_to_text(line_data.to_string().as_bytes());

                            for d in colored_data.lines {
                                for x in d.spans {
                                    result_line_spans.push(x);
                                }
                            }
                            result_str_elements.push(
                                str_line_style
                                    .paint(line_data.to_string().to_string())
                                    .to_string(),
                            );
                        } else {
                            let color_strip_data = get_ansi_strip_str(&line_data);
                            result_line_spans
                                .push(Span::styled(line_data.to_string(), tui_line_style));
                            result_str_elements.push(
                                str_line_style
                                    .paint(color_strip_data.to_string().to_string())
                                    .to_string(),
                            );
                        }
                    }
                    _ => {
                        line_data = ansi::get_ansi_strip_str(&value);
                        let color_strip_data = get_ansi_strip_str(&line_data)
                            .trim_end_matches('\n')
                            .to_string();
                        result_line_spans.push(Span::styled(line_data.to_string(), tui_line_style));
                        result_str_elements.push(
                            str_line_style
                                .paint(color_strip_data.to_string().to_string())
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
        if self.is_line_number {
            let line_number = line_number + 1;
            let header_width = self.header_width;
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
                &gen_counter_str(self.is_color, line_number as usize, header_width, diff_type),
            );
        }

        if self.is_batch {
            PrintElementData::String(result_str.trim_end_matches('\n').to_string())
        } else {
            PrintElementData::Line(result_line)
        }
    }

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

    // set is reverse.
    pub fn set_reverse(&mut self, is_reverse: bool) -> &mut Self {
        self.is_reverse = is_reverse;
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
            _ => {}
        }
    }

    if is_batch {
        PrintData::Strings(strings)
    } else {
        PrintData::Lines(lines)
    }
}

///
fn gen_counter_str(
    is_color: bool,
    counter: usize,
    header_width: usize,
    diff_type: DifferenceType,
) -> String {
    let mut counter_str = counter.to_string();
    let mut seprator = " | ".to_string();
    let mut prefix_width = 0;
    let mut suffix_width = 0;

    if is_color {
        let style: ansi_term::Style = match diff_type {
            DifferenceType::Same => ansi_term::Style::default().fg(COLOR_BATCH_LINE_NUMBER_DEFAULT),
            DifferenceType::Add => ansi_term::Style::default().fg(COLOR_BATCH_LINE_NUMBER_ADD),
            DifferenceType::Rem => ansi_term::Style::default().fg(COLOR_BATCH_LINE_NUMBER_REM),
        };
        counter_str = style.paint(counter_str).to_string();
        seprator = style.paint(seprator).to_string();
        prefix_width = style.prefix().to_string().len();
        suffix_width = style.suffix().to_string().len();
    }

    let width = header_width + prefix_width + suffix_width;
    format!("{counter_str:>width$}{seprator}")
}
