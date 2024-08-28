// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    style::{Style, Color},
    prelude::{Line, Margin},
    symbols,
    symbols::scrollbar,
    widgets::{Paragraph, Wrap, Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

#[derive(Clone)]
pub struct WatchArea<'a> {
    ///
    area: tui::layout::Rect,

    ///
    pub data: Vec<Line<'a>>,

    ///
    position: i16,

    ///
    lines: i16,

    /// is enable border
    border: bool,

    // is hideen header pane
    hide_header: bool,

    /// is enable scroll bar
    scroll_bar: bool,
}

/// Watch Area Object Trait
impl<'a> WatchArea<'a> {
    ///
    pub fn new() -> Self {
        //! new Self
        Self {
            area: tui::layout::Rect::new(0, 0, 0, 0),

            data: vec![Line::from("")],

            position: 0,

            lines: 0,

            border: false,

            hide_header: false,

            scroll_bar: false,
        }
    }

    ///
    pub fn set_area(&mut self, area: tui::layout::Rect) {
        self.area = area;
    }

    ///
    pub fn get_area_size(&mut self) -> i16 {
        let height = self.area.height as i16;

        return height
    }

    ///
    pub fn update_output(&mut self, data: Vec<Line<'a>>) {
        self.data = data;
    }

    ///
    pub fn set_border(&mut self, border: bool) {
        self.border = border;
    }

    ///
    pub fn set_scroll_bar(&mut self, scroll_bar: bool) {
        self.scroll_bar = scroll_bar;
    }

    ///
    pub fn set_hide_header(&mut self, hide_header: bool) {
        self.hide_header = hide_header;
    }

    ///
    pub fn draw(&mut self, frame: &mut Frame) {
        // declare variables
        let pane_block: Block<'_>;

        // check is border enable
        if self.border {
            if self.hide_header {
                pane_block = Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_set(
                        symbols::border::Set {
                            top_right: symbols::line::NORMAL.horizontal_down,
                            ..symbols::border::PLAIN
                        }
                    );
            } else {
                pane_block = Block::default()
                    .borders(Borders::TOP | Borders::RIGHT)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_set(
                        symbols::border::Set {
                            top_right: symbols::line::NORMAL.horizontal_down,
                            ..symbols::border::PLAIN
                        }
                    );
            }
        } else {
            pane_block = Block::default()
        }

        //
        let block = Paragraph::new(self.data.clone())
            .style(Style::default())
            .block(pane_block)
            .wrap(Wrap { trim: false })
            .scroll((self.position as u16, 0));

        // get self.lines
        let mut pane_width: u16 = self.area.width as u16;
        if self.border {
            pane_width = pane_width - 1;
        }

        self.lines = block.line_count(pane_width) as i16;

        frame.render_widget(block, self.area);

        // render scrollbar
        if self.border && self.scroll_bar && self.lines > self.area.height as i16 {
            let mut scrollbar_state: ScrollbarState = ScrollbarState::default()
                .content_length(self.lines as usize - self.area.height as usize)
                .position(self.position as usize);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            self.area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    ///
    pub fn scroll_up(&mut self, num: i16) {
        self.position = std::cmp::max(0, self.position - num);
    }

    ///
    pub fn scroll_down(&mut self, num: i16) {
        let mut height: u16 = self.area.height;
        if self.border {
            if !self.hide_header {
                height = height - 1;
            }
        }

        if self.lines > height as i16 {
            self.position = std::cmp::min(self.position + num, self.lines - height as i16);
        }
    }

    ///
    pub fn scroll_home(&mut self) {
        self.position = 0;
    }

    ///
    pub fn scroll_end(&mut self) {
        let mut height: u16 = self.area.height;
        if self.border {
            if !self.hide_header {
                height = height - 1;
            }
        }

        if self.lines > height as i16 {
            self.position = self.lines - height as i16;
        }
    }

}
