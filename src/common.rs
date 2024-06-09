// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use chrono::Local;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::prelude::*;

use tui::layout::{Constraint, Direction, Layout, Rect};

// local module
use crate::exec::CommandResult;

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiffMode {
    Disable,
    Watch,
    Line,
    Word,
}

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Output,
    Stdout,
    Stderr,
}

///
pub fn now_str() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
}

/// logging result data to log file(_logpath).
pub fn logging_result(_logpath: &str, result: &CommandResult) -> Result<(), Box<dyn Error>> {
    // try open logfile
    let mut logfile = match OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(_logpath)
    {
        Err(why) => return Err(Box::new(why)),
        Ok(file) => file,
    };

    // create logline
    let logdata = serde_json::to_string(&result.export_data())?;

    // write log
    // TODO(blacknon): warning出てるので対応
    _ = writeln!(logfile, "{logdata}");

    Ok(())
}

///
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect_with_size(height: u16, width: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - height) / 2),
                Constraint::Length(height),
                Constraint::Length((r.height - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - width) / 2),
                Constraint::Length(width),
                Constraint::Length((r.width - width) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
