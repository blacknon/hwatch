// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: リファクタリング

use tui::{
    prelude::{Line, Margin},
    style::{Color, Style, Styled},
    symbols::{self, scrollbar},
    text::Span,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame
};

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct WatchArea<'a> {
    /// ratatui::layout::Rect. The area to draw the widget in.
    area: tui::layout::Rect,

    /// Original data.
    pub data: Vec<Line<'a>>,

    /// Wrapped data.
    wrap_data: Vec<Line<'a>>,

    /// search keyword.
    keyword: String,

    /// search keyword is regex flag.
    keyword_is_regex: bool,

    /// search keyword positions. (line_number, keyword_start, keyword_end)
    keyword_position: Vec<(usize, usize, usize)>,

    /// selected keyword index.
    selected_keyword: i16,

    /// line number.
    pub is_line_number: bool,

    /// scroll position.
    position: i16,

    /// wrap_data line count.
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

            wrap_data: vec![Line::from("")],

            keyword: String::from(""),

            keyword_is_regex: false,

            keyword_position: vec![],

            selected_keyword: -1,

            is_line_number: false,

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
        // update data
        self.data = data;

        // update wrap data
        self.wrap_data = wrap_utf8_lines(&self.data, self.area.width as usize);

        if self.keyword.len() > 0 {
            // update keyword position
            self.keyword_position = get_keyword_positions(&self.wrap_data, &self.keyword, self.keyword_is_regex, self.is_line_number);
        }
    }

    ///
    pub fn update_wrap(&mut self) {
        // update wrap data
        self.wrap_data = wrap_utf8_lines(&self.data, self.area.width as usize);

        if self.keyword.len() > 0 {
            // update keyword position
            self.keyword_position = get_keyword_positions(&self.wrap_data, &self.keyword, self.keyword_is_regex, self.is_line_number);
        }
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
    pub fn set_keyword(&mut self, keyword: String, is_regex: bool) {
        self.keyword = keyword;
        self.keyword_is_regex = is_regex;
        self.selected_keyword = -1;

        if self.keyword_position.len() > self.selected_keyword as usize {
            self.selected_keyword = -1;
        }

        // update wrap data
        self.wrap_data = wrap_utf8_lines(&self.data, self.area.width as usize);

        if self.keyword.len() > 0 {
            // update keyword position
            self.keyword_position = get_keyword_positions(&self.wrap_data, &self.keyword, self.keyword_is_regex, self.is_line_number);

            self.next_keyword();
        } else {
            self.keyword_position = vec![];
        }
    }

    ///
    pub fn reset_keyword(&mut self) {
        self.keyword = String::from("");
        self.keyword_is_regex = false;
        self.keyword_position = vec![];
        self.selected_keyword = -1;
    }

    ///
    pub fn previous_keyword(&mut self) {
        // update keyword position
        self.keyword_position = get_keyword_positions(&self.wrap_data, &self.keyword, self.keyword_is_regex, self.is_line_number);

        if self.selected_keyword > 0 {
            self.selected_keyword -= 1;
        } else if self.selected_keyword == 0 {
            self.selected_keyword = self.keyword_position.len() as i16 - 1;
        }

        if self.keyword_position.len() - 1 < self.selected_keyword as usize {
            self.selected_keyword = self.keyword_position.len() as i16 - 1;
        }

        // get selected keyword position
        let position = self.keyword_position[self.selected_keyword as usize];

        // scroll move
        self.scroll_move(position.0 as i16);
    }

    ///
    pub fn next_keyword(&mut self) {
        // update keyword position
        self.keyword_position = get_keyword_positions(&self.wrap_data, &self.keyword, self.keyword_is_regex, self.is_line_number);

        // get selected keyword position
        if self.keyword_position.len() < self.selected_keyword as usize {
            self.selected_keyword = -1;
        }

        if self.selected_keyword < self.keyword_position.len() as i16 - 1 {
            self.selected_keyword += 1;
        } else if self.selected_keyword == self.keyword_position.len() as i16 - 1 {
            self.selected_keyword = 0;
        }

        if self.keyword_position.len() >= self.selected_keyword as usize + 1 && self.selected_keyword >= 0 {
            let position: (usize, usize, usize) = self.keyword_position[self.selected_keyword as usize];

            // scroll move
            self.scroll_move(position.0 as i16);
        }
    }

    ///
    pub fn draw(&mut self, frame: &mut Frame) {
        // set highlight style
        let highlight_style = Style::new().fg(Color::Black).bg(Color::Yellow);
        let selected_highlight_style = Style::new().fg(Color::Black).bg(Color::Cyan);

        let block_data = highlight_text(&self.wrap_data, self.keyword_position.clone(), self.selected_keyword, selected_highlight_style, highlight_style);

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
        let block = Paragraph::new(block_data)
            .style(Style::default())
            .block(pane_block)
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
            self.position = std::cmp::min(self.position + num, self.lines - height as i16 - 1);
        }
    }

    ///
    pub fn scroll_home(&mut self) {
        self.position = 0;
    }

    ///
    pub fn scroll_end(&mut self) {
        let mut height: i16 = self.area.height as i16;
        if self.border {
            if !self.hide_header {
                height = height - 1;
            }
        }

        if self.lines > height {
            self.position = self.lines - height - 1;
        }
    }

    ///
    pub fn scroll_move(&mut self, position: i16) {
        let mut height: i16 = self.area.height as i16;
        if self.border {
            if !self.hide_header {
                height = height - 1;
            }
        }

        let start = self.position;
        let end = std::cmp::min(self.position + height, self.lines);

        if start < position && position < end {
            return
        } else if start > position {
            self.position = position;
        } else if end < position + 1 {
            self.position = position - height + 1;
        }
    }

}

///
fn get_keyword_positions(lines: &Vec<Line>, keyword: &str, is_regex: bool, is_line_number: bool) -> Vec<(usize, usize, usize)> {
    let mut ignore_head_count = 0;
    if is_line_number {
        let num_count = lines.len().to_string().len();
        ignore_head_count = num_count + 3; // ^<number>` | `
    }

    let re = if is_regex {
        Some(Regex::new(keyword).expect("Invalid regex pattern"))
    } else {
        None
    };

    let mut hits = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let combined_text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();
        let combined_text = if is_line_number {
            combined_text[ignore_head_count..].to_string()
        } else {
            combined_text
        };

        if let Some(re) = &re {
            for mat in re.find_iter(&combined_text) {
                hits.push((line_index, mat.start() + ignore_head_count, mat.end() + ignore_head_count));
            }
        } else {
            let mut start_position = 0;

            while let Some(pos) = combined_text[start_position..].find(keyword) {
                let match_start = start_position + pos;
                let match_end = match_start + keyword.len();
                hits.push((line_index, match_start + ignore_head_count, match_end + ignore_head_count));
                start_position = match_end;
            }
        }
    }

    hits
}

///
fn wrap_utf8_lines<'a>(lines: &Vec<Line>, width: usize) -> Vec<Line<'a>> {
    let mut wrapped_lines = Vec::new();

    for line in lines {
        let mut current_line = Line::default();
        let mut current_width = 0;

        for span in &line.spans {
            let words = span.content.split_inclusive(|c| c == ' ' || c == '\u{00a0}' || c == '\u{200b}');
            for word in words {
                let word_width = unicode_width::UnicodeWidthStr::width(word);

                if current_width + word_width > width {
                    if !current_line.spans.is_empty() {
                        wrapped_lines.push(current_line);
                    }
                    current_line = Line::default();
                    current_width = 0;

                    if word_width > width {
                        let mut grapheme_iter = UnicodeSegmentation::graphemes(word, true);
                        while let Some(grapheme) = grapheme_iter.next() {
                            let grapheme_width = unicode_width::UnicodeWidthStr::width(grapheme);
                            if current_width + grapheme_width > width {
                                wrapped_lines.push(current_line);
                                current_line = Line::default();
                                current_width = 0;
                            }
                            let style = span.style().clone();
                            current_line.spans.push(Span::styled(grapheme.to_string(), style));
                            current_width += grapheme_width;
                        }
                        continue;
                    }
                }

                current_line.spans.push(Span::styled(word.to_string(), span.style().clone()));
                current_width += word_width;
            }
        }

        // 最後の行を追加
        if !current_line.spans.is_empty() {
            wrapped_lines.push(current_line);
        }
    }

    wrapped_lines
}

///
fn highlight_text<'a>(lines: &'a Vec<Line>, positions: Vec<(usize, usize, usize)>, selected_keyword: i16, selected_highlight_style: Style, highlight_style: Style) -> Vec<Line<'a>> {
    let mut new_lines = Vec::new();
    let mut current_count:i16 = 0;

    for (i, line) in lines.iter().enumerate() {
        let mut new_spans = Vec::new();
        let mut current_pos = 0;

        // Get the highlighted position of the corresponding keyword for this line
        let line_hits: Vec<(usize, usize, usize)> = positions
            .iter()
            .filter(|(line_index, _, _)| *line_index == i)
            .cloned()
            .collect();

        // Process each Span to generate a new Span
        for span in &line.spans {
            let span_text = span.content.as_ref().to_string();
            let span_start = current_pos;
            let span_end = current_pos + span_text.len();

            // Processing when the highlight range spans Span
            if !line_hits.is_empty() {
                let mut last_pos = 0;
                for (_, start_position, end_position) in line_hits.iter() {
                    // Ignore if the hit is after the current span
                    if *start_position >= span_end {
                        continue;
                    }

                    // Calculating highlight_start and highlight_end
                    let highlight_start = (*start_position).saturating_sub(span_start); // 値が負にならないように調整
                    let highlight_end = (*end_position).min(span_end).saturating_sub(span_start);

                    if highlight_start > last_pos {
                        new_spans.push(Span::styled(
                            span_text[last_pos..highlight_start].to_string(),
                            span.style,
                        ));
                    }

                    let text_str: String = span_text[highlight_start..highlight_end].to_string();

                    if text_str.len() > 0 {
                        if current_count == selected_keyword {
                            new_spans.push(Span::styled(
                                text_str,
                                selected_highlight_style,
                            ));
                        } else {
                            new_spans.push(Span::styled(
                                text_str,
                                highlight_style,
                            ));
                        }
                        current_count += 1;
                    }

                    last_pos = highlight_end;
                }

                if last_pos < span_text.len() {
                    new_spans.push(Span::styled(
                        span_text[last_pos..].to_string(),
                        span.style,
                    ));
                }
            } else {
                // Apply existing style to parts that are not highlights
                new_spans.push(Span::styled(span_text.clone(), span.style));
            }

            current_pos += span_text.len();
        }

        new_lines.push(Line::from(new_spans));
    }

    new_lines
}
