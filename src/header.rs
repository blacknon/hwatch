// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::iter;
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Wrap},
    Frame,
};

// local module
use exec::Result;
use view::{ActiveArea, DiffMode, OutputMode};

//const
const POSITION_X_HELP_TEXT: usize = 56;
const WIDTH_TEXT_INTERVAL: usize = 19;

pub struct HeaderArea<'a> {
    ///
    area: tui::layout::Rect,

    ///
    color: bool,

    ///
    diff_mode: DiffMode,

    ///
    interval: f64,

    ///
    ///
    data: Vec<Spans<'a>>,

    ///
    output_mode: OutputMode,
}

impl<'a> HeaderArea<'a> {
    pub fn new() -> Self {
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),
            color: false,
            diff_mode: DiffMode::Disable,
            interval: ::DEFAULT_INTERVAL,
            data: vec![Spans::from("")],
            output_mode: OutputMode::Output,
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn update(&mut self, result: Result, active: &ActiveArea) {
        // init data
        self.data = vec![];

        // help message
        let help_message = "Display help with h key!";

        // HeaderArea Width
        let width = self.area.width as usize;

        // Value for width calculation.
        let command_width = width - (WIDTH_TEXT_INTERVAL + POSITION_X_HELP_TEXT);
        let timestamp_width =
            width - (WIDTH_TEXT_INTERVAL + command_width + help_message.len()) + 1;
        let filter_keyword_width = width - POSITION_X_HELP_TEXT - 2;
        let filter_keyword: String = format!("{:wid$}", "", wid = filter_keyword_width);

        // Get the data to display at header.
        let interval = format!("{:.3}", self.interval);
        let command = result.command;
        let timestamp = result.timestamp;
        let status = result.status;

        // Set output type value
        let value_output: String;
        match self.output_mode {
            OutputMode::Output => value_output = "Output".to_string(),
            OutputMode::Stdout => value_output = "Stdout".to_string(),
            OutputMode::Stderr => value_output = "Stderr".to_string(),
        }

        // Set Active Area value
        let value_active: String;
        match active {
            ActiveArea::History => value_active = "history".to_string(),
            ActiveArea::Watch => value_active = "watch".to_string(),
        }

        // Set Diff mode value
        let value_diff: String;
        match self.diff_mode {
            DiffMode::Disable => value_diff = "None".to_string(),
            DiffMode::Watch => value_diff = "Watch".to_string(),
            DiffMode::Line => value_diff = "Line".to_string(),
        }

        // Set Color
        let command_color: Color;
        let is_enable_color: Color;
        match status {
            true => command_color = Color::Green,
            false => command_color = Color::Red,
        }
        match self.color {
            true => is_enable_color = Color::Green,
            false => is_enable_color = Color::Reset,
        }

        // Create 1st line.
        self.data.push(Spans::from(vec![
            Span::raw("Every "),
            Span::styled(
                format!("{:>wid$}", interval, wid = 9),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("s: "),
            Span::styled(
                format!("{:wid$}", command, wid = command_width),
                Style::default().fg(command_color),
            ),
            Span::styled(
                format!("Display help with h key!"),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(
                format!("{:>wid$}", timestamp, wid = timestamp_width),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Create 2nd line
        self.data.push(Spans::from(vec![
            // filter keyword
            Span::styled(":", Style::default().fg(Color::Yellow)),
            Span::styled(filter_keyword, Style::default().fg(Color::Yellow)),
            // Color flag
            Span::styled("Color: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", self.color, wid = 5),
                Style::default().fg(is_enable_color),
            ),
            Span::raw(" "),
            // Output Type
            Span::styled("Output: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_output, wid = 6),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" "),
            // Active Area
            Span::styled("Active: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_active, wid = 7),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" "),
            // Diff Type
            Span::styled("Diff: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_diff, wid = 5),
                Style::default().fg(Color::Magenta),
            ),
        ]));
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let block = Paragraph::new(self.data.clone()).wrap(Wrap { trim: true });
        frame.render_widget(block, self.area);
    }
}
