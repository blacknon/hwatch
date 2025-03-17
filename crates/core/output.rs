// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: watch diffのハイライトについて、旧watchと同様のパターンとは別により見やすいハイライトを実装する
//       => Watch(Word)とかにするか？？
//       => すぐには出来なさそう？なので、0.3.15にて対応とする？
// TODO: 行・文字差分数の取得を行うための関数を作成する(ここじゃないかも？？)

// modules
// use std::borrow::Cow;
// use std::fmt::Write;
use std::sync::{Arc, Mutex};
use tui::prelude::Line;

// local const
use crate::DEFAULT_TAB_SIZE;

// local module
use crate::common::OutputMode;
use crate::exec::CommandResult;
use hwatch_diffmode::{expand_line_tab, DiffMode, DiffModeOptions};

pub struct Printer {
    // diff mode.
    diff_mode: Arc<Mutex<Box<dyn DiffMode>>>,

    // output mode.
    output_mode: OutputMode,

    // batch mode.
    is_batch: bool,

    // diffmode options.
    options: DiffModeOptions,

    // is reverse text.
    is_reverse: bool,

    // tab size.
    tab_size: u16,
}

impl Printer {
    pub fn new(diffmode: Arc<Mutex<Box<dyn DiffMode>>>) -> Self {
        Self {
            diff_mode: diffmode,
            output_mode: OutputMode::Output,
            is_batch: false,
            options: DiffModeOptions::new(),
            is_reverse: false,
            tab_size: DEFAULT_TAB_SIZE,
        }
    }

    ///
    pub fn get_watch_text<'a>(
        &mut self,
        dest: &CommandResult,
        src: &CommandResult,
    ) -> Vec<Line<'a>> {
        // set new text(text_dst)
        let mut text_dest = match self.output_mode {
            OutputMode::Output => (*dest).get_output(),
            OutputMode::Stdout => (*dest).get_stdout(),
            OutputMode::Stderr => (*dest).get_stderr(),
        };
        text_dest = expand_line_tab(&text_dest, self.tab_size);
        if !self.options.get_color() {
            text_dest = hwatch_ansi::escape_ansi(&text_dest);
        }

        // set old text(text_src)
        let mut text_src = match self.output_mode {
            OutputMode::Output => (*src).get_output(),
            OutputMode::Stdout => (*src).get_stdout(),
            OutputMode::Stderr => (*src).get_stderr(),
        };
        text_src = expand_line_tab(&text_src, self.tab_size);
        if !self.options.get_color() {
            text_src = hwatch_ansi::escape_ansi(&text_src);
        }

        let result: Vec<Line<'_>>;

        let mut diff_mode = self.diff_mode.lock().unwrap();

        // set diff mode options
        diff_mode.set_option(self.options);

        // create diff
        result = diff_mode.generate_watch_diff(&text_dest, &text_src);

        if self.is_reverse {
            result.into_iter().rev().collect()
        } else {
            result
        }
    }

    ///
    pub fn get_batch_text(&mut self, dest: &CommandResult, src: &CommandResult) -> Vec<String> {
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

        let result: Vec<String>;

        let mut diff_mode = self.diff_mode.lock().unwrap();

        // set diff mode options
        diff_mode.set_option(self.options);

        // create diff
        result = diff_mode.generate_batch_diff(&text_dest, &text_src);

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
        self.options.set_color(is_color);
        self
    }

    /// set line number.
    pub fn set_line_number(&mut self, is_line_number: bool) -> &mut Self {
        self.options.set_line_number(is_line_number);
        self
    }

    // set is reverse.
    pub fn set_reverse(&mut self, is_reverse: bool) -> &mut Self {
        self.is_reverse = is_reverse;
        self
    }

    /// set diff mode.
    pub fn set_only_diffline(&mut self, is_only_diffline: bool) -> &mut Self {
        self.options.set_only_diffline(is_only_diffline);
        self
    }

    /// set tab size.
    pub fn set_tab_size(&mut self, tab_size: u16) -> &mut Self {
        self.tab_size = tab_size;
        self
    }
}
