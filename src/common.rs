// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use chrono::Local;
use serde_json::Deserializer;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;

use tui::layout::{Constraint, Direction, Layout, Rect};

// local module
use crate::exec::{CommandResult, CommandResultData};

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
    date.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub enum LoadLogfileError {
    LogfileEmpty,
    LoadFileError(std::io::Error),
    JsonParseError(serde_json::Error),
}

pub fn load_logfile(
    log_path: &str,
    is_compress: bool,
) -> Result<Vec<CommandResult>, LoadLogfileError> {
    // fileのサイズをチェックするし、0だった場合commandが必須であるエラーを返す
    if let Ok(metadata) = fs::metadata(log_path) {
        if metadata.len() == 0 {
            return Err(LoadLogfileError::LogfileEmpty);
        }
    }

    // load log file
    let file = match File::open(log_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(LoadLogfileError::LoadFileError(e));
        }
    };

    // create file reader
    let reader = BufReader::new(file);

    // create stream use by Deserializer
    let stream = Deserializer::from_reader(reader).into_iter::<CommandResultData>();

    // load and add data.
    let mut result_data = vec![];
    for log_data in stream {
        match log_data {
            Ok(data) => result_data.push(data.generate_result(is_compress)),
            Err(e) => {
                return Err(LoadLogfileError::JsonParseError(e));
            }
        }
    }

    Ok(result_data)
}

/// logging result data to log file(_logpath).
pub fn logging_result(_logpath: &str, result: &CommandResult) -> Result<(), Box<dyn Error>> {
    // try open logfile
    let mut logfile = match OpenOptions::new().create(true).append(true).open(_logpath) {
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
