// Copyright (c) 2026 Blacknon. All rights reserved.
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
use tui::style::Color;

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
                Constraint::Length(r.height.saturating_sub(height) / 2),
                Constraint::Length(height),
                Constraint::Length(r.height.saturating_sub(height) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(r.width.saturating_sub(width) / 2),
                Constraint::Length(width),
                Constraint::Length(r.width.saturating_sub(width) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn parse_ansi_color(value: &str) -> Result<Color, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("value is empty".to_string());
    }

    let lower = trimmed.to_ascii_lowercase();

    let named = match lower.as_str() {
        "default" | "reset" => Some(Color::Reset),
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
    };

    if let Some(color) = named {
        return Ok(color);
    }

    if let Some(hex) = lower.strip_prefix('#') {
        if hex.len() == 6 {
            let r =
                u8::from_str_radix(&hex[0..2], 16).map_err(|_| "invalid hex value".to_string())?;
            let g =
                u8::from_str_radix(&hex[2..4], 16).map_err(|_| "invalid hex value".to_string())?;
            let b =
                u8::from_str_radix(&hex[4..6], 16).map_err(|_| "invalid hex value".to_string())?;
            return Ok(Color::Rgb(r, g, b));
        }
    }

    let rgb_source = lower.strip_prefix("rgb:").unwrap_or(lower.as_str());
    if rgb_source.contains(',') {
        let parts: Vec<&str> = rgb_source.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0]
                .trim()
                .parse::<u8>()
                .map_err(|_| "invalid rgb value".to_string())?;
            let g = parts[1]
                .trim()
                .parse::<u8>()
                .map_err(|_| "invalid rgb value".to_string())?;
            let b = parts[2]
                .trim()
                .parse::<u8>()
                .map_err(|_| "invalid rgb value".to_string())?;
            return Ok(Color::Rgb(r, g, b));
        }
    }

    if let Ok(idx) = lower.parse::<u16>() {
        if idx <= 255 {
            return Ok(Color::Indexed(idx as u8));
        }
        return Err("color index must be 0-255".to_string());
    }

    Err(format!(
        "invalid ANSI color value: '{trimmed}'. Use a color name, 0-255, #RRGGBB, or R,G,B."
    ))
}
