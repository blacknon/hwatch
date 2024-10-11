// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: commandの表示を単色ではなく、Syntax highlightしたカラーリングに書き換える(v0.3.9)
// TODO: input内容の表示
// TODO: 幅調整系の数字をconstにする(生数字で雑計算だとわけわからん)

use tui::{
    style::{Color, Modifier, Style},
    text::Span,

    widgets::Paragraph,
    Frame,
    prelude::Line,
};

// local module
use crate::app::{ActiveArea, InputMode};
use crate::common::{DiffMode, OutputMode};
use crate::exec::CommandResult;

//const
// const POSITION_X_HELP_TEXT: usize = 47;
const WIDTH_TEXT_INTERVAL: usize = 15;
const WIDTH_TIMESTAMP: usize = 23; // "20XX-XX-XX XX:XX:XX.XXX".len() .. 19

#[derive(Clone)]
pub struct HeaderArea<'a> {
    ///
    pub area: tui::layout::Rect,

    ///
    interval: f64,

    ///
    command: String,

    ///
    timestamp: String,

    ///
    exec_status: bool,

    ///
    data: Vec<Line<'a>>,

    ///
    line_number: bool,

    ///
    reverse: bool,

    ///
    ansi_color: bool,

    ///
    active_area: ActiveArea,

    ///
    diff_mode: DiffMode,

    ///
    is_only_diffline: bool,

    ///
    banner: String,

    ///
    output_mode: OutputMode,

    ///
    input_mode: InputMode,

    ///
    input_prompt: String,

    ///
    pub input_text: String,
}

/// Header Area Object Trait
impl<'a> HeaderArea<'a> {
    pub fn new(interval: f64) -> Self {
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),

            interval,

            command: "".to_string(),
            timestamp: "".to_string(),
            exec_status: true,

            data: vec![Line::from("")],
            ansi_color: false,
            line_number: false,
            reverse: false,
            banner: "".to_string(),

            active_area: ActiveArea::History,

            diff_mode: DiffMode::Disable,
            is_only_diffline: false,

            output_mode: OutputMode::Output,

            input_mode: InputMode::None,
            input_prompt: "".to_string(),
            input_text: "".to_string(),
        }
    }

    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    pub fn set_active_area(&mut self, active: ActiveArea) {
        self.active_area = active;
    }

    pub fn set_output_mode(&mut self, mode: OutputMode) {
        self.output_mode = mode;
    }

    pub fn set_line_number(&mut self, line_number: bool) {
        self.line_number = line_number;
    }

    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }

    pub fn set_banner(&mut self, banner: String) {
        self.banner = banner;
    }

    pub fn set_ansi_color(&mut self, ansi_color: bool) {
        self.ansi_color = ansi_color;
    }

    pub fn set_current_result(&mut self, result: CommandResult) {
        self.command = result.command;
        self.timestamp = result.timestamp;
        self.exec_status = result.status;
    }

    pub fn set_interval(&mut self, interval: f64) {
        self.interval = interval;
    }

    pub fn set_diff_mode(&mut self, diff_mode: DiffMode) {
        self.diff_mode = diff_mode;
    }

    pub fn set_is_only_diffline(&mut self, is_only_diffline: bool) {
        self.is_only_diffline = is_only_diffline
    }

    pub fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }

    ///
    pub fn update(&mut self) {
        // init data
        self.data = vec![];

        // HeaderArea Width
        let width = self.area.width as usize;

        // Value for width calculation.
        let command_width: usize;
        let timestamp_width: usize;
        if WIDTH_TEXT_INTERVAL + (1 + self.banner.len() + 1 + WIDTH_TIMESTAMP) < width {
            command_width = width - 2 - 1 - WIDTH_TEXT_INTERVAL - self.banner.len() - 1 - WIDTH_TIMESTAMP;
            timestamp_width = WIDTH_TIMESTAMP;
        } else {
            command_width = 0;
            timestamp_width = 0;
        }

        // filter keyword.
        let filter_keyword_width = if width > ((self.banner.len() + 20) + 2 + 14) {
            // width - POSITION_X_HELP_TEXT - 2 - 14
            // length("[Number] [Color] [Output] [history] [Line(Only)]") = 48
            // length("[Number] [Color] [Reverse] [Output] [history] [Line(Only)]") = 58
            width - 59
        } else {
            0
        };
        let filter_keyword = format!("{:wid$}", self.input_text, wid = filter_keyword_width);
        let filter_keyword_style: Style;

        if self.input_text.is_empty() {
            match self.input_mode {
                InputMode::Filter => self.input_prompt = "/".to_string(),
                InputMode::RegexFilter => self.input_prompt = "*".to_string(),

                _ => self.input_prompt = " ".to_string(),
            }

            filter_keyword_style = Style::default().fg(Color::Gray);
        } else {
            filter_keyword_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);
        }

        // Get the data to display at header.
        let interval = format!("{:.3}", self.interval);

        // Set Number flag value
        let value_number: Span = match self.line_number {
            true => Span::styled(
                "Number".to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
            ),
            false => Span::styled("Number".to_string(), Style::default().fg(Color::Reset)),
        };

        // Set Color flag value
        let value_color: Span = match self.ansi_color {
            true => Span::styled(
                "Color".to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
            ),
            false => Span::styled("Color".to_string(), Style::default().fg(Color::Reset)),
        };

        // Set Reverse flag value
        let value_reverse: Span = match self.reverse {
            true => Span::styled(
                "Reverse".to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
            ),
            false => Span::styled("Reverse".to_string(), Style::default().fg(Color::Reset)),
        };

        // Set Output type value
        let value_output = match self.output_mode {
            OutputMode::Output => "Output".to_string(),
            OutputMode::Stdout => "Stdout".to_string(),
            OutputMode::Stderr => "Stderr".to_string(),
        };

        // Set Active Area value
        let value_active = match self.active_area {
            ActiveArea::History => "History".to_string(),
            ActiveArea::Watch => "Watch".to_string(),
        };

        // Set IsOnlyDiffline value
        let value_only_diffline = if self.is_only_diffline { "(Only)" } else { "" };

        // Set Diff mode value
        let value_diff = match self.diff_mode {
            DiffMode::Disable => "None".to_string(),
            DiffMode::Watch => "Watch".to_string(),
            DiffMode::Line => "Line".to_string() + value_only_diffline,
            DiffMode::Word => "Word".to_string() + value_only_diffline,
        };

        // Set Color
        let command_color = match self.exec_status {
            true => Color::Green,
            false => Color::Red,
        };

        // Create 1st line.
        self.data.push(Line::from(vec![
            Span::raw("Every "),
            Span::styled(
                format!("{:>wid$}", interval, wid = 9),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("s:"),
            Span::styled(
                format!("{:wid$}", self.command, wid = command_width),
                Style::default().fg(command_color),
            ),
            Span::raw(" "),
            Span::styled(
                self.banner.clone(),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{:>wid$}", self.timestamp, wid = timestamp_width),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Create 2nd line
        self.data.push(Line::from(vec![
            // filter keyword
            Span::styled(self.input_prompt.clone(), Style::default().fg(Color::Gray)),
            Span::styled(filter_keyword, filter_keyword_style),
            // Line number flag
            Span::raw("["), value_number, Span::raw("]"), Span::raw(" "),
            // Color flag
            Span::raw("["), value_color, Span::raw("]"), Span::raw(" "),
            // Reverse flag
            Span::raw("["), value_reverse, Span::raw("]"), Span::raw(" "),
            // Output Type
            Span::raw("["),
            // Span::styled("Output:", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_output, wid = 6),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::REVERSED),
            ),
            Span::raw("]"),
            Span::raw(" "),
            // Active Area
            Span::raw("["),
            // Span::styled("Active:", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_active, wid = 7),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::REVERSED),
            ),
            Span::raw("]"),
            Span::raw(" "),
            // Diff Type
            Span::raw("["),
            // Span::styled("Diff: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_diff, wid = 10),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::REVERSED),
            ),
            Span::raw("]"),
        ]));
    }

    ///
    pub fn draw(&mut self, frame: &mut Frame) {
        let block = Paragraph::new(self.data.clone());
        frame.render_widget(block, self.area);
    }
}
