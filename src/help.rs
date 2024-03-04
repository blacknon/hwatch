// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    prelude::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct HelpWindow<'a> {
    ///
    data: Vec<Line<'a>>,

    ///
    position: i16,
}

/// History Area Object Trait
impl<'a> HelpWindow<'a> {
    pub fn new() -> Self {
        let data = gen_help_text();

        Self { data, position: 0 }
    }

    ///
    pub fn draw(&mut self, f: &mut Frame) {
        let title = "help";

        let size = f.size();
        let area = centered_rect(60, 50, size);

        // create block.
        let block = Paragraph::new(self.data.clone())
            .style(Style::default().fg(Color::LightGreen))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray).bg(Color::Reset)),
            )
            .scroll((self.position as u16, 0));

        f.render_widget(Clear, area);
        f.render_widget(block, area);
    }

    ///
    pub fn scroll_up(&mut self, num: i16) {
        if 0 <= self.position - num {
            self.position -= num
        }
    }

    ///
    pub fn scroll_down(&mut self, num: i16) {
        // get area data size
        let data_size = self.data.len() as i16;

        if data_size > self.position + num {
            self.position += num
        }
    }
}

///
fn gen_help_text<'a>() -> Vec<Line<'a>> {
    // set help messages.
    let text = vec![
        Line::from(" - [h] key   ... show this help message."),
        // toggle
        Line::from(" - [c] key   ... toggle color mode."),
        Line::from(" - [n] key   ... toggle line number."),
        Line::from(" - [d] key   ... switch diff mode at None, Watch, Line, and Word mode. "),
        Line::from(" - [t] key   ... toggle ui (history pane & header both on/off). "),
        Line::from(" - [Bkspace] ... toggle history pane. "),
        Line::from(" - [m] key   ... toggle mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key."),
        // exit hwatch
        Line::from(" - [q] key   ... exit hwatch."),
        // change diff
        Line::from(" - [0] key   ... disable diff."),
        Line::from(" - [1] key   ... switch Watch type diff."),
        Line::from(" - [2] key   ... switch Line type diff."),
        Line::from(" - [3] key   ... switch Word type diff."),
        // change output
        Line::from(" - [F1] key  ... change output mode as stdout."),
        Line::from(" - [F2] key  ... change output mode as stderr."),
        Line::from(" - [F3] key  ... change output mode as output(stdout/stderr set.)"),
        // change interval
        Line::from(" - [+] key ... Increase interval by .5 seconds."),
        Line::from(" - [-] key ... Decrease interval by .5 seconds."),
        // change use area
        Line::from(" - [Tab] key ... toggle current area at history or watch."),
        // filter text input
        Line::from(" - [/] key   ... filter history by string."),
        Line::from(" - [*] key   ... filter history by regex."),
        Line::from(" - [ESC] key ... unfiltering."),
    ];

    text
}

///
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
