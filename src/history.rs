// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

pub struct HistoryArea<'a> {
    area: tui::layout::Rect,
    data: Vec<Spans<'a>>,
    scroll_position: u16,
}
