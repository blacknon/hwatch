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
use tui::style::Color;

// local const
use crate::DEFAULT_TAB_SIZE;

// local module
use crate::common::OutputMode;
use crate::exec::CommandResult;
use hwatch_diffmode::{DiffMode, DiffModeOptions};

#[path = "output_render.rs"]
mod render;
use self::render::{
    maybe_reverse_lines, maybe_reverse_strings, prepare_batch_text, prepare_watch_text,
};

pub struct PaneContent {
    pub lines: Vec<Line<'static>>,
    pub is_line_number: bool,
    pub is_line_diff_head: bool,
}

pub enum WatchRenderData {
    SinglePane(PaneContent),
}

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
    pub fn get_watch_data(&mut self, dest: &CommandResult, src: &CommandResult) -> WatchRenderData {
        let text_dest = prepare_watch_text(
            dest,
            self.output_mode,
            self.tab_size,
            self.options.get_color(),
        );
        let text_src = prepare_watch_text(
            src,
            self.output_mode,
            self.tab_size,
            self.options.get_color(),
        );

        let mut diff_mode = self.diff_mode.lock().unwrap();

        // set diff mode options
        diff_mode.set_option(self.options);
        let is_line_diff_head = diff_mode.get_support_only_diffline();

        // create diff
        let result = diff_mode.generate_watch_diff(&text_dest, &text_src);

        let lines = maybe_reverse_lines(result, self.is_reverse);

        WatchRenderData::SinglePane(PaneContent {
            lines,
            is_line_number: self.options.get_line_number(),
            is_line_diff_head,
        })
    }

    ///
    pub fn get_batch_text(&mut self, dest: &CommandResult, src: &CommandResult) -> Vec<String> {
        let text_dest = prepare_batch_text(dest, self.output_mode);
        let text_src = prepare_batch_text(src, self.output_mode);

        let mut diff_mode = self.diff_mode.lock().unwrap();

        // set diff mode options
        diff_mode.set_option(self.options);

        // create diff
        let result = diff_mode.generate_batch_diff(&text_dest, &text_src);

        maybe_reverse_strings(result, self.is_reverse)
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

    pub fn set_ignore_spaceblock(&mut self, ignore_spaceblock: bool) -> &mut Self {
        self.options.set_ignore_spaceblock(ignore_spaceblock);
        self
    }

    /// set tab size.
    pub fn set_tab_size(&mut self, tab_size: u16) -> &mut Self {
        self.tab_size = tab_size;
        self
    }

    /// set watch diff highlight colors (no-op placeholder)
    pub fn set_watch_diff_colors(&mut self, _fg: Option<Color>, _bg: Option<Color>) -> &mut Self {
        // Currently colors are handled by diffmode implementations; keep this
        // method as a no-op to preserve API compatibility until diffmode
        // supports runtime color changes.
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::OutputMode;
    use crate::diffmode_line::DiffModeAtLineDiff;
    use crate::exec::CommandResult;

    fn line_diff_printer() -> Printer {
        let diff_mode: Arc<Mutex<Box<dyn DiffMode>>> =
            Arc::new(Mutex::new(Box::new(DiffModeAtLineDiff::new())));
        Printer::new(diff_mode)
    }

    #[test]
    fn get_batch_text_omits_unchanged_lines_when_only_diffline_is_enabled() {
        let mut printer = line_diff_printer();
        let before = CommandResult::default().set_output(b"same\nbefore\n".to_vec());
        let after = CommandResult::default().set_output(b"same\nafter\n".to_vec());

        let lines = printer
            .set_output_mode(OutputMode::Output)
            .set_color(false)
            .set_only_diffline(true)
            .get_batch_text(&after, &before);

        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|line| !line.contains("same")));
        assert!(lines.iter().any(|line| line.contains("before")));
        assert!(lines.iter().any(|line| line.contains("after")));
    }

    #[test]
    fn get_batch_text_treats_whitespace_only_changes_as_equal_when_ignored() {
        let mut printer = line_diff_printer();
        let before = CommandResult::default().set_output(b"alpha  beta\n".to_vec());
        let after = CommandResult::default().set_output(b"alpha   beta\n".to_vec());

        let ignored = printer
            .set_output_mode(OutputMode::Output)
            .set_color(false)
            .set_only_diffline(false)
            .set_ignore_spaceblock(true)
            .get_batch_text(&after, &before);

        let mut strict_printer = line_diff_printer();
        let strict = strict_printer
            .set_output_mode(OutputMode::Output)
            .set_color(false)
            .set_only_diffline(false)
            .set_ignore_spaceblock(false)
            .get_batch_text(&after, &before);

        assert_eq!(ignored, vec!["alpha   beta".to_string()]);
        assert_eq!(strict.len(), 2);
    }
}
