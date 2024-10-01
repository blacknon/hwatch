// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use tui::{
    prelude::{Line, Margin}, style::{Color, Style, Styled}, symbols::{self, scrollbar}, text::Span, widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap}, Frame
};

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;


// TODO： 検索キーワードの保持とその位置情報の保持を追加
// TODO: 検索キーワードの指定や削除のmethodを追加

#[derive(Clone)]
pub struct WatchArea<'a> {
    ///
    area: tui::layout::Rect,

    ///
    pub data: Vec<Line<'a>>,

    ///
    area_data: Vec<Line<'a>>,

    ///
    keyword: String,

    ///
    keyword_is_regex: bool,

    ///
    keyword_position: Vec<(usize, usize, usize)>,

    ///
    selected_keyword: i16,

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

            area_data: vec![Line::from("")],

            keyword: String::from(""),

            keyword_is_regex: false,

            keyword_position: vec![],

            selected_keyword: 0,

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
    pub fn set_keyword(&mut self, keyword: String, is_regex: bool) {
        self.keyword = keyword;
        self.keyword_is_regex = is_regex;
    }

    pub fn reset_keyword(&mut self) {
        self.keyword = String::from("");
        self.keyword_is_regex = false;
        self.keyword_position = vec![];
        self.selected_keyword = 0;
    }

    ///
    pub fn previous_keyword(&mut self) {
        if self.selected_keyword > 0 {
            self.selected_keyword -= 1;
        }
    }

    ///
    pub fn next_keyword(&mut self) {
        if self.selected_keyword < self.keyword_position.len() as i16 - 1 {
            self.selected_keyword += 1;
        }
    }

    ///
    pub fn draw(&mut self, frame: &mut Frame) {
        // TODO: ColorをStyleで渡すように変更
        // TODO: 現在選択されているキーワードは別のSytleを指定するよう変更
        // TODO: wrap_utf8_linesかhighlight_textのどちらか、あるいは両方で既存のStyleがリセットされているようなので、修正
        // create highlight data for keyword
        // check is keyword

        let highlight_color = Color::Yellow;
        let wrap_data = wrap_utf8_lines(&self.data, self.area.width as usize);
        if self.keyword.len() > 0 {
            self.keyword_position = get_keyword_positions(&wrap_data, &self.keyword, self.keyword_is_regex);
        }
        let block_data = highlight_text(&wrap_data, self.keyword_position.clone(), highlight_color);

        // // create block data
        // let start = self.selected_keyword as usize;
        // let end: usize = std::cmp::min(start + self.area.height as usize, wrapped_lines.len());
        // let block_data: Vec<Line> = wrapped_lines[start..end].to_vec();

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


///
fn get_keyword_positions(lines: &Vec<Line>, keyword: &str, is_regex: bool) -> Vec<(usize, usize, usize)> {
    // 正規表現をコンパイルする（正規表現モードの場合のみ）
    let re = if is_regex {
        Some(Regex::new(keyword).expect("Invalid regex pattern"))
    } else {
        None
    };

    // キーワード検索の結果を格納するVec
    let mut hits = Vec::new();

    // forでLineのチェックをしていく
    for (line_index, line) in lines.iter().enumerate() {
        // 各 Line 内のすべての Span を結合して、一つの文字列にする
        let combined_text: String = line.spans.iter().map(|span| span.content.as_ref()).collect();

        if let Some(re) = &re {
            // 正規表現にマッチする部分を検索
            for mat in re.find_iter(&combined_text) {
                hits.push((line_index, mat.start(), mat.end()));
            }
        } else {
            // プレーンテキスト検索の場合
            let mut start_position = 0;

            // テキスト内でキーワードが見つかるたびにその位置を記録
            while let Some(pos) = combined_text[start_position..].find(keyword) {
                let match_start = start_position + pos;
                let match_end = match_start + keyword.len();
                hits.push((line_index, match_start, match_end));
                start_position = match_end; // 見つかった位置の次から再検索
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

        // 各 Line の Span を処理して、スペースやゼロ幅スペースで分割
        for span in &line.spans {
            let words = span.content.split_inclusive(|c| c == ' ' || c == '\u{00a0}' || c == '\u{200b}');
            for word in words {
                let word_width = unicode_width::UnicodeWidthStr::width(word);

                if current_width + word_width > width {
                    // 幅を超えた場合は現在の行を追加し、新しい行を開始
                    if !current_line.spans.is_empty() {
                        wrapped_lines.push(current_line);
                    }
                    current_line = Line::default();
                    current_width = 0;

                    // 単語が幅を超える場合、グラフェム単位で折り返し
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

/// 複数の Span にまたがるキーワードを正しくハイライトするための関数
/// ハイライトされない部分は既存のスタイルを保持
fn highlight_text<'a>(lines: &'a Vec<Line>, positions: Vec<(usize, usize, usize)>, highlight_color: Color) -> Vec<Line<'a>> {
    let mut new_lines = Vec::new(); // 新しい Vec<Line> を生成

    for (i, line) in lines.iter().enumerate() {
        let mut new_spans = Vec::new();
        let mut current_pos = 0;

        // この行に対する該当するキーワードのハイライト位置を取得
        let line_hits: Vec<(usize, usize, usize)> = positions
            .iter()
            .filter(|(line_index, _, _)| *line_index == i)
            .cloned()
            .collect();

        // 各 Span を処理して、新しい Span を生成
        for span in &line.spans {
            let span_text = span.content.as_ref().to_string();
            let span_start = current_pos;
            let span_end = current_pos + span_text.len();

            // ハイライト範囲が Span にかかっている場合の処理
            if !line_hits.is_empty() {
                let mut last_pos = 0;
                for (_, start_position, end_position) in line_hits.iter() {
                    // ヒット箇所が現在のスパンの後なら無視
                    if *start_position >= span_end {
                        continue;
                    }

                    // highlight_startおよびhighlight_endの計算を実施
                    let highlight_start = (*start_position).saturating_sub(span_start); // 値が負にならないように調整
                    let highlight_end = (*end_position).min(span_end).saturating_sub(span_start);

                    // ハイライト前の部分（既存のスタイルを適用）
                    if highlight_start > last_pos {
                        new_spans.push(Span::styled(
                            span_text[last_pos..highlight_start].to_string(),
                            span.style,
                        ));
                    }

                    // ハイライト部分
                    new_spans.push(Span::styled(
                        span_text[highlight_start..highlight_end].to_string(),
                        Style::default().bg(highlight_color).fg(Color::Black),
                    ));

                    // ハイライト後の部分
                    last_pos = highlight_end;
                }

                // 残りの部分を処理
                if last_pos < span_text.len() {
                    new_spans.push(Span::styled(
                        span_text[last_pos..].to_string(),
                        span.style,
                    ));
                }
            } else {
                // ハイライトのない部分（既存のスタイルを適用）
                new_spans.push(Span::styled(span_text.clone(), span.style));
            }

            current_pos += span_text.len();
        }

        new_lines.push(Line::from(new_spans));
    }

    new_lines
}
