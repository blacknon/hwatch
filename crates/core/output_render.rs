// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crate::common::OutputMode;
use crate::exec::CommandResult;
use hwatch_diffmode::expand_line_tab;
use tui::prelude::Line;

pub(super) fn prepare_watch_text(
    result: &CommandResult,
    output_mode: OutputMode,
    tab_size: u16,
    use_color: bool,
) -> String {
    let mut text = match output_mode {
        OutputMode::Output => result.get_output(),
        OutputMode::Stdout => result.get_stdout(),
        OutputMode::Stderr => result.get_stderr(),
    };
    text = expand_line_tab(&text, tab_size);
    if !use_color {
        text = hwatch_ansi::escape_ansi(&text);
    }
    text
}

pub(super) fn prepare_batch_text(result: &CommandResult, output_mode: OutputMode) -> String {
    match output_mode {
        OutputMode::Output => result.get_output(),
        OutputMode::Stdout => result.get_stdout(),
        OutputMode::Stderr => result.get_stderr(),
    }
}

pub(super) fn maybe_reverse_lines(
    lines: Vec<Line<'static>>,
    is_reverse: bool,
) -> Vec<Line<'static>> {
    if is_reverse {
        lines.into_iter().rev().collect()
    } else {
        lines
    }
}

pub(super) fn maybe_reverse_strings(lines: Vec<String>, is_reverse: bool) -> Vec<String> {
    if is_reverse {
        lines.into_iter().rev().collect()
    } else {
        lines
    }
}
