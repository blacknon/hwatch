// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: diff時のカラーコードについても対応する
//       (1行ごとにansi4tui::bytes_to_textに放り込む方式？行最後のカラーコードを保持して続きを記述することでdiffでも対応出来るかも？)

// modules
use difference::{Changeset, Difference};
use std::{cmp, option::Option};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

pub fn get_watch_diff<'a>(old: &str, new: &str) -> Vec<Spans<'a>> {
    let mut result = vec![];

    // output to vector
    let mut old_vec: Vec<&str> = old.lines().collect();
    let mut new_vec: Vec<&str> = new.lines().collect();

    // get max line
    let max_line = cmp::max(old_vec.len(), new_vec.len());

    // for diff lines
    for i in 0..max_line {
        // push empty line
        if old_vec.len() <= i {
            old_vec.push("");
        }
        if new_vec.len() <= i {
            new_vec.push("");
        }

        // TODO: 1行ごとの出力用関数呼び出し
        let line_data = get_watch_diff_line(old_vec[i], new_vec[i]);
        result.push(line_data);
    }

    return result;
}

fn get_watch_diff_line<'a>(old_line: &str, new_line: &str) -> Spans<'a> {
    // If the contents are the same line.
    if old_line == new_line {
        return Spans::from(String::from(new_line));
    }

    // Decompose lines by character.
    let mut old_line_chars: Vec<char> = old_line.chars().collect();
    let mut new_line_chars: Vec<char> = new_line.chars().collect();

    let space: char = ' ';
    let max_char = cmp::max(old_line_chars.len(), new_line_chars.len());

    let mut _result = vec![];
    for x in 0..max_char {
        if old_line_chars.len() <= max_char {
            old_line_chars.push(space);
        }

        if new_line_chars.len() <= max_char {
            new_line_chars.push(space);
        }

        if old_line_chars[x] != new_line_chars[x] {
            // add span
            _result.push(Span::styled(
                new_line_chars[x].to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::REVERSED),
            ));
        } else {
            // add span
            _result.push(Span::styled(
                new_line_chars[x].to_string(),
                Style::default(),
            ));
        }
    }

    return Spans::from(_result);
}
