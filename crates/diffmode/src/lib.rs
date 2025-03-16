// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

extern crate ratatui as tui;

// local crate
// extern crate hwatch_ansi as ansi;

use ansi_term::Colour;
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

// type
pub type DiffModeMutex = Arc<Mutex<Box<dyn DiffMode>>>;

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
pub enum DifferenceType {
    Same,
    Add,
    Rem,
}

// NOTE:
//  以下のコードは、output.rsで前処理コードとして後で追加する
//     // tab expand dest
//     let mut text_dest = dest.to_string();
//     if !self.is_batch {
//         text_dest = expand_line_tab(dest, self.tab_size);
//
//         if !self.is_color {
//             text_dest = ansi::escape_ansi(&text_dest);
//         }
//     }
//     let text_dest_bytes = text_dest.as_bytes().to_vec();
//
//     // tab expand src
//     let mut text_src = src.to_string();
//     if !self.is_batch {
//         text_src = expand_line_tab(src, self.tab_size);
//
//         if !self.is_color {
//             text_src = ansi::escape_ansi(&text_src);
//         }
//     }
//     let text_src_bytes = text_src.as_bytes().to_vec();

// TODO: headerで出力する文字列取得用のMethodを追加する
// TODO: output onlyに対応しているかどうかを出力するMethodを追加する

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

/// DiffModeOptions
#[derive(Debug, Clone, Copy)]
pub struct DiffModeOptions {
    //
    color: bool,

    //
    line_number: bool,

    //
    word_highlight: bool,

    //
    only_diffline: bool,
}

impl DiffModeOptions {
    pub fn new() -> Self {
        Self {
            color: false,
            line_number: false,
            word_highlight: false,
            only_diffline: false,
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

    pub fn get_word_highlight(&self) -> bool {
        self.word_highlight
    }

    pub fn set_word_highlight(&mut self, word_highlight: bool) {
        self.word_highlight = word_highlight;
    }

    pub fn get_only_diffline(&self) -> bool {
        self.only_diffline
    }

    pub fn set_only_diffline(&mut self, only_diffline: bool) {
        self.only_diffline = only_diffline;
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
        return OutputVecData::Strings(strings);
    } else {
        return OutputVecData::Lines(lines);
    };
}
