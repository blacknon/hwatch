// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: commandの表示を単色ではなく、Syntax highlightしたカラーリングに書き換える(v0.3.0)
// TODO: input内容の表示
// TODO: 幅調整系の数字をconstにする(生数字で雑計算だとわけわからん)

use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

// local module
use crate::app::{ActiveArea, DiffMode, InputMode, OutputMode};
use crate::exec::CommandResult;

// local const
use crate::DEFAULT_INTERVAL;

//const
const POSITION_X_HELP_TEXT: usize = 53;
const WIDTH_TEXT_INTERVAL: usize = 19;

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
    data: Vec<Spans<'a>>,

    ///
    line_number: bool,

    ///
    ansi_color: bool,

    ///
    active_area: ActiveArea,

    ///
    diff_mode: DiffMode,

    ///
    is_only_diffline: bool,

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
    pub fn new() -> Self {
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),

            interval: DEFAULT_INTERVAL,

            command: "".to_string(),
            timestamp: "".to_string(),
            exec_status: true,

            data: vec![Spans::from("")],
            ansi_color: false,
            line_number: false,

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

    pub fn update(&mut self) {
        // init data
        self.data = vec![];

        // help message
        const HELP_MESSAGE: &str = "Display help with h key!";

        // HeaderArea Width
        let width = self.area.width as usize;

        // Value for width calculation.
        let command_width: usize;
        let timestamp_width: usize;
        if WIDTH_TEXT_INTERVAL + POSITION_X_HELP_TEXT < width {
            command_width = width - (WIDTH_TEXT_INTERVAL + POSITION_X_HELP_TEXT);
            timestamp_width =
                width - (WIDTH_TEXT_INTERVAL + command_width + HELP_MESSAGE.len()) + 1;
        } else {
            command_width = 0;
            timestamp_width = 0;
        }

        // filter keyword.
        let filter_keyword_width = if width > (POSITION_X_HELP_TEXT + 2 + 14) {
            width - POSITION_X_HELP_TEXT - 2 - 14
        } else {
            0
        };
        let filter_keyword = format!("{:wid$}", self.input_text, wid = filter_keyword_width);
        let filter_keyword_style: Style;

        if self.input_text.is_empty() {
            match self.input_mode {
                InputMode::Filter => self.input_prompt = "/".to_string(),
                InputMode::RegexFilter => self.input_prompt = "*".to_string(),

                _ => self.input_prompt = "".to_string(),
            }

            filter_keyword_style = Style::default().fg(Color::Gray);
        } else {
            filter_keyword_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);
        }

        // Get the data to display at header.
        let interval = format!("{:.3}", self.interval);

        // Set number flag value
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

        // Set output type value
        let value_output = match self.output_mode {
            OutputMode::Output => "Output".to_string(),
            OutputMode::Stdout => "Stdout".to_string(),
            OutputMode::Stderr => "Stderr".to_string(),
        };

        // Set Active Area value
        let value_active = match self.active_area {
            ActiveArea::History => "history".to_string(),
            ActiveArea::Watch => "watch".to_string(),
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
        self.data.push(Spans::from(vec![
            Span::raw("Every "),
            Span::styled(
                format!("{:>wid$}", interval, wid = 9),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("s: "),
            Span::styled(
                format!("{:wid$}", self.command, wid = command_width),
                Style::default().fg(command_color),
            ),
            Span::styled(
                HELP_MESSAGE.to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(
                format!("{:>wid$}", self.timestamp, wid = timestamp_width),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Create 2nd line
        self.data.push(Spans::from(vec![
            // filter keyword
            Span::styled(self.input_prompt.clone(), Style::default().fg(Color::Gray)),
            Span::styled(filter_keyword, filter_keyword_style),
            // Line number flag
            Span::raw("["),
            value_number,
            Span::raw("]"),
            Span::raw(" "),
            // Color flag
            Span::raw("["),
            value_color,
            Span::raw("]"),
            Span::raw(" "),
            // Output Type
            Span::raw("["),
            Span::styled("Output:", Style::default().add_modifier(Modifier::BOLD)),
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
            Span::styled("Active: ", Style::default().add_modifier(Modifier::BOLD)),
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
            Span::styled("Diff: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:wid$}", value_diff, wid = 10),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::REVERSED),
            ),
            Span::raw("]"),
        ]));
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let block = Paragraph::new(self.data.clone());
        frame.render_widget(block, self.area);
    }
}
