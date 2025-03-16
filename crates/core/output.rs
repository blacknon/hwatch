// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: watch diffのハイライトについて、旧watchと同様のパターンとは別により見やすいハイライトを実装する
//       => Watch(Word)とかにするか？？
//       => すぐには出来なさそう？なので、0.3.15にて対応とする？
// TODO: 行・文字差分数の取得を行うための関数を作成する(ここじゃないかも？？)

// modules
use std::borrow::Cow;
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use tui::prelude::Line;

// local const
use crate::DEFAULT_TAB_SIZE;

// local module
use crate::common::OutputMode;
use crate::exec::CommandResult;
use hwatch_diffmode::{expand_line_tab, DiffMode, DiffModeOptions};

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
    diff_mode: Arc<Mutex<Box<dyn DiffMode>>>,

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
    pub fn new(diffmode: Arc<Mutex<Box<dyn DiffMode>>>) -> Self {
        Self {
            diff_mode: diffmode,
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

        let mut result = vec![];

        {
            // create diff mode options
            let mut diff_mode_options = DiffModeOptions::new();
            diff_mode_options.set_color(self.is_color);
            diff_mode_options.set_line_number(self.is_line_number);
            diff_mode_options.set_word_highlight(self.is_word_highlight);
            diff_mode_options.set_only_diffline(self.is_only_diffline);

            let mut diff_mode = self.diff_mode.lock().unwrap();

            // set diff mode options
            diff_mode.set_option(diff_mode_options);

            // create diff
            result = diff_mode.generate_watch_diff(&text_dest, &text_src);
        }

        if self.is_reverse {
            result.into_iter().rev().collect()
        } else {
            result
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
        text_dest = expand_line_tab(&text_dest, self.tab_size);
        if self.is_color {
            text_dest = hwatch_ansi::escape_ansi(&text_dest);
        }

        // set old text(text_src)
        let mut text_src = match self.output_mode {
            OutputMode::Output => (*src).get_output(),
            OutputMode::Stdout => (*src).get_stdout(),
            OutputMode::Stderr => (*src).get_stderr(),
        };
        text_src = expand_line_tab(&text_src, self.tab_size);
        if self.is_color {
            text_src = hwatch_ansi::escape_ansi(&text_src);
        }

        let mut result = vec![];

        {
            // create diff mode options
            let mut diff_mode_options = DiffModeOptions::new();
            diff_mode_options.set_color(self.is_color);
            diff_mode_options.set_line_number(self.is_line_number);
            diff_mode_options.set_word_highlight(self.is_word_highlight);
            diff_mode_options.set_only_diffline(self.is_only_diffline);

            let mut diff_mode = self.diff_mode.lock().unwrap();

            // set diff mode options
            diff_mode.set_option(diff_mode_options);

            // create diff
            result = diff_mode.generate_batch_diff(&text_dest, &text_src);
        }

        if self.is_reverse {
            result.into_iter().rev().collect()
        } else {
            result
        }
    }

    /// set diff mode.
    pub fn set_diff_mode(&mut self, diff_mode: Arc<Mutex<Box<dyn DiffMode>>>) -> &mut Self {
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
