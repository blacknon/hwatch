// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

extern crate ratatui as tui;

// local crate
// extern crate hwatch_ansi as ansi;

use ansi_term::Colour;
use std::ffi::c_char;
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::{borrow::Cow, vec};

use tui::{prelude::Line, style::Color};

// const
pub const COLOR_BATCH_LINE_NUMBER_DEFAULT: Colour = Colour::Fixed(240);
pub const COLOR_BATCH_LINE_NUMBER_ADD: Colour = Colour::RGB(56, 119, 120);
pub const COLOR_BATCH_LINE_NUMBER_REM: Colour = Colour::RGB(118, 0, 0);
pub const COLOR_BATCH_LINE_ADD: Colour = Colour::Green;
pub const COLOR_BATCH_LINE_REM: Colour = Colour::Red;
pub const COLOR_BATCH_LINE_REVERSE_FG: Colour = Colour::White;
pub const COLOR_WATCH_LINE_NUMBER_DEFAULT: Color = Color::DarkGray;
pub const COLOR_WATCH_LINE_NUMBER_ADD: Color = Color::Rgb(56, 119, 120);
pub const COLOR_WATCH_LINE_NUMBER_REM: Color = Color::Rgb(118, 0, 0);
pub const COLOR_WATCH_LINE_ADD: Color = Color::Green;
pub const COLOR_WATCH_LINE_REM: Color = Color::Red;
pub const COLOR_WATCH_LINE_REVERSE_FG: Color = Color::White;
pub const PLUGIN_ABI_VERSION: u32 = 2;
pub const PLUGIN_ABI_VERSION_V1: u32 = 1;
pub const PLUGIN_OUTPUT_BATCH: u32 = 0;
pub const PLUGIN_OUTPUT_WATCH: u32 = 1;

// type
pub type DiffModeMutex = Arc<Mutex<Box<dyn DiffMode>>>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginSlice {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginOwnedBytes {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginDiffRequestV1 {
    pub dest: PluginSlice,
    pub src: PluginSlice,
    pub output_kind: u32,
    pub color: bool,
    pub line_number: bool,
    pub only_diffline: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginDiffRequest {
    pub dest: PluginSlice,
    pub src: PluginSlice,
    pub output_kind: u32,
    pub color: bool,
    pub line_number: bool,
    pub only_diffline: bool,
    pub ignore_spaceblock: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginMetadata {
    pub abi_version: u32,
    pub supports_only_diffline: bool,
    pub plugin_name: *const c_char,
    pub header_text: *const c_char,
}

// OutputVecData is ...
pub enum OutputVecData<'a> {
    Lines(Vec<Line<'a>>),
    Strings(Vec<String>),
}

// OutputVecElementData is ...
pub enum OutputVecElementData<'a> {
    Line(Line<'a>),
    String(String),
    None(),
}

// DifferenceType is ...
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifferenceType {
    Same,
    Add,
    Rem,
}

pub struct DiffRow<'a> {
    pub watch_line: Line<'a>,
    pub batch_line: String,
    pub line_number: Option<usize>,
    pub diff_type: DifferenceType,
}

// TODO: headerで出力する文字列取得用のMethodを追加する
// TODO: output onlyに対応しているかどうかを出力するMethodを追加する

pub trait StringExt {
    fn expand_tabs(&self, tab_size: u16) -> Cow<'_, str>;
}

impl<T> StringExt for T
where
    T: AsRef<str>,
{
    fn expand_tabs(&self, tab_size: u16) -> Cow<'_, str> {
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

/// DiffModeOptions
#[derive(Debug, Clone, Copy)]
pub struct DiffModeOptions {
    //
    color: bool,

    //
    line_number: bool,

    //
    only_diffline: bool,

    //
    ignore_spaceblock: bool,
}

impl DiffModeOptions {
    pub fn new() -> Self {
        Self {
            color: false,
            line_number: false,
            only_diffline: false,
            ignore_spaceblock: false,
        }
    }

    pub fn get_color(&self) -> bool {
        self.color
    }

    pub fn set_color(&mut self, color: bool) {
        self.color = color;
    }

    pub fn get_line_number(&self) -> bool {
        self.line_number
    }

    pub fn set_line_number(&mut self, line_number: bool) {
        self.line_number = line_number;
    }

    pub fn get_only_diffline(&self) -> bool {
        self.only_diffline
    }

    pub fn set_only_diffline(&mut self, only_diffline: bool) {
        self.only_diffline = only_diffline;
    }

    pub fn get_ignore_spaceblock(&self) -> bool {
        self.ignore_spaceblock
    }

    pub fn set_ignore_spaceblock(&mut self, ignore_spaceblock: bool) {
        self.ignore_spaceblock = ignore_spaceblock;
    }
}

impl Default for DiffModeOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// DiffMode
pub trait DiffMode {
    // generate and return diff watch window result.
    fn generate_watch_diff(&mut self, dest: &str, src: &str) -> Vec<Line<'static>>;

    // generate and return diff batch result.
    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String>;

    // get header's text
    fn get_header_text(&self) -> String;

    // get support only diffline
    fn get_support_only_diffline(&self) -> bool;

    // オプション指定用function
    fn set_option(&mut self, options: DiffModeOptions);
}

/// get_option add DiffMode
pub trait DiffModeExt: DiffMode {
    fn get_option<T: 'static>(&self) -> DiffModeOptions;

    fn get_header_width<T: 'static>(&self) -> usize;
}

pub fn expand_line_tab(data: &str, tab_size: u16) -> String {
    let mut result_vec: Vec<String> = vec![];
    for d in data.lines() {
        let l = d.expand_tabs(tab_size).to_string();
        result_vec.push(l);
    }

    result_vec.join("\n")
}

pub fn normalize_space_blocks(data: &str) -> String {
    let mut normalized = String::with_capacity(data.len());
    let mut in_spaceblock = false;

    for ch in data.chars() {
        if ch == '\n' {
            normalized.push('\n');
            in_spaceblock = false;
            continue;
        }

        if ch.is_whitespace() {
            if !in_spaceblock {
                normalized.push(' ');
                in_spaceblock = true;
            }
        } else {
            normalized.push(ch);
            in_spaceblock = false;
        }
    }

    normalized
}

pub fn text_eq_ignoring_space_blocks(left: &str, right: &str, enabled: bool) -> bool {
    if !enabled {
        return left == right;
    }

    normalize_space_blocks(left) == normalize_space_blocks(right)
}

pub fn gen_counter_str(
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

pub fn expand_output_vec_element_data(
    is_batch: bool,
    data: Vec<OutputVecElementData>,
) -> OutputVecData {
    let mut lines = Vec::new();
    let mut strings = Vec::new();

    for element in data {
        match element {
            OutputVecElementData::Line(line) => {
                lines.push(line);
            }
            OutputVecElementData::String(string) => {
                strings.push(string);
            }
            _ => {}
        }
    }

    if is_batch {
        OutputVecData::Strings(strings)
    } else {
        OutputVecData::Lines(lines)
    }
}

pub fn render_diff_rows_as_watch<'a>(
    rows: Vec<DiffRow<'a>>,
    is_line_number: bool,
    header_width: usize,
) -> Vec<Line<'a>> {
    rows.into_iter()
        .map(|mut row| {
            if is_line_number {
                let style = tui::style::Style::default().fg(match row.diff_type {
                    DifferenceType::Same => COLOR_WATCH_LINE_NUMBER_DEFAULT,
                    DifferenceType::Add => COLOR_WATCH_LINE_NUMBER_ADD,
                    DifferenceType::Rem => COLOR_WATCH_LINE_NUMBER_REM,
                });
                let prefix = match row.line_number {
                    Some(line_number) => format!("{line_number:>header_width$} | "),
                    None => format!("{:>header_width$} | ", ""),
                };
                row.watch_line
                    .spans
                    .insert(0, tui::text::Span::styled(prefix, style));
            }
            row.watch_line
        })
        .collect()
}

pub fn render_diff_rows_as_batch<'a>(
    rows: Vec<DiffRow<'a>>,
    is_color: bool,
    is_line_number: bool,
    header_width: usize,
) -> Vec<String> {
    rows.into_iter()
        .map(|row| {
            if is_line_number {
                match row.line_number {
                    Some(line_number) => format!(
                        "{}{}",
                        gen_counter_str(is_color, line_number, header_width, row.diff_type),
                        row.batch_line
                    ),
                    None => format!("{:>header_width$} | {}", "", row.batch_line),
                }
            } else {
                row.batch_line
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui::text::Span;

    #[test]
    fn expand_tabs_replaces_tabs_with_spaces() {
        assert_eq!("a\tb".expand_tabs(4), "a   b");
    }

    #[test]
    fn expand_tabs_with_zero_tab_size_removes_tab_padding() {
        assert_eq!("a\tb".expand_tabs(0), "ab");
    }

    #[test]
    fn diff_mode_options_round_trip_each_flag() {
        let mut options = DiffModeOptions::new();
        options.set_color(true);
        options.set_line_number(true);
        options.set_only_diffline(true);
        options.set_ignore_spaceblock(true);

        assert!(options.get_color());
        assert!(options.get_line_number());
        assert!(options.get_only_diffline());
        assert!(options.get_ignore_spaceblock());
    }

    #[test]
    fn expand_line_tab_expands_each_line_independently() {
        assert_eq!(expand_line_tab("a\tb\n12\tc", 4), "a   b\n12  c");
    }

    #[test]
    fn normalize_space_blocks_collapses_runs_per_line() {
        assert_eq!(
            normalize_space_blocks("a  b\t\tc\n  d    e\n"),
            "a b c\n d e\n"
        );
    }

    #[test]
    fn text_eq_ignoring_space_blocks_matches_equivalent_text() {
        assert!(text_eq_ignoring_space_blocks("a  b", "a   b", true));
        assert!(text_eq_ignoring_space_blocks("a\tb", "a    b", true));
        assert!(!text_eq_ignoring_space_blocks("ab", "a b", true));
        assert!(!text_eq_ignoring_space_blocks("a  b", "a   b", false));
    }

    #[test]
    fn gen_counter_str_without_color_is_plain_text() {
        assert_eq!(
            gen_counter_str(false, 12, 4, DifferenceType::Same),
            "  12 | "
        );
    }

    #[test]
    fn gen_counter_str_with_color_wraps_output_in_ansi_sequences() {
        let counter = gen_counter_str(true, 7, 3, DifferenceType::Add);

        assert!(counter.contains("\u{1b}["));
        assert!(counter.contains("7"));
        assert!(counter.ends_with(" | \u{1b}[0m"));
    }

    #[test]
    fn expand_output_vec_element_data_returns_batch_strings() {
        let output = expand_output_vec_element_data(
            true,
            vec![
                OutputVecElementData::String("first".to_string()),
                OutputVecElementData::Line(Line::from(vec![Span::raw("ignored")])),
                OutputVecElementData::String("second".to_string()),
            ],
        );

        match output {
            OutputVecData::Strings(strings) => {
                assert_eq!(strings, vec!["first".to_string(), "second".to_string()]);
            }
            OutputVecData::Lines(_) => panic!("expected string output"),
        }
    }

    #[test]
    fn expand_output_vec_element_data_returns_watch_lines() {
        let output = expand_output_vec_element_data(
            false,
            vec![
                OutputVecElementData::String("ignored".to_string()),
                OutputVecElementData::Line(Line::from("watch line")),
            ],
        );

        match output {
            OutputVecData::Lines(lines) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0].spans[0].content.as_ref(), "watch line");
            }
            OutputVecData::Strings(_) => panic!("expected line output"),
        }
    }
}
