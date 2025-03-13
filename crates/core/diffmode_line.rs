// Copyright (c) 2025 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    prelude::Line,
    style::{Color, Style},
    text::Span,
};

use hwatch_ansi as ansi;
use hwatch_diffmode::{gen_counter_str, DiffMode, DiffModeExt, DiffModeOptions, DifferenceType};

pub struct DiffModeAtLineDiff {
    options: DiffModeOptions,
}

impl DiffModeAtLineDiff {
    pub fn new() -> Self {
        Self {
            options: DiffModeOptions::new(),
        }
    }
}

impl DiffMode for DiffModeAtLineDiff {
    fn generate_watch_diff(&self, dest: &str, src: &str) -> Vec<Line<'static>> {
        //
        let mut result = Vec::new();

        return result;
    }

    fn generate_batch_diff(&self, dest: &str, src: &str) -> Vec<String> {
        //
        let mut result = Vec::new();

        return result;
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
}
