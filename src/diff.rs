// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: diff時のカラーコードについても対応する
//       (1行ごとにansi4tui::bytes_to_textに放り込む方式？行最後のカラーコードを保持して続きを記述することでdiffでも対応出来るかも？)
//       - watch ... 同居(ansiで対応)
//       - line ... 差分のある行はansiを削除(強制でdiff colorに書き換え)
//       - word ... 差分のある行はansiを削除(強制でdiff colorに書き換え)

// modules
use difference::{Changeset, Difference};
use std::cmp;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

// watch diff
// ==========

///
pub fn get_watch_diff<'a>(color: bool, old: &str, new: &str) -> Vec<Spans<'a>> {
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
        let line_data: Spans;
        match color {
            false => line_data = get_watch_diff_line(old_vec[i], new_vec[i]),
            true => line_data = get_watch_diff_line_with_ansi(old_vec[i], new_vec[i]),
        }
        result.push(line_data);
    }

    return result;
}

///
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
                Style::default().add_modifier(Modifier::REVERSED),
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

///
fn get_watch_diff_line_with_ansi<'a>(old_line: &str, new_line: &str) -> Spans<'a> {
    // TODO: 書く. 差分発生箇所をANSIで記述して、それをansi4tuiに渡して変換する方式とする
    return Spans::from("");
}

// line diff
// ==========

///
pub fn get_line_diff<'a>(color: bool, old: &str, new: &str) -> Vec<Spans<'a>> {
    // Create changeset
    let Changeset { diffs, .. } = Changeset::new(old, new, "\n");

    // create result
    let mut result = vec![];

    for i in 0..diffs.len() {
        match diffs[i] {
            // Same line.
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
                    let data = Spans::from(format!("   {}\n", line));
                    result.push(data);
                }
            }

            // Add line.
            Difference::Add(ref diff_data) => {
                for line in diff_data.lines() {
                    let data = Spans::from(Span::styled(
                        format!("+  {}\n", line),
                        Style::default().fg(Color::Green),
                    ));
                    result.push(data);
                }
            }

            // Remove line.
            Difference::Rem(ref diff_data) => {
                for line in diff_data.lines() {
                    let data = Spans::from(Span::styled(
                        format!("-  {}\n", line),
                        Style::default().fg(Color::Red),
                    ));
                    result.push(data);
                }
            }
        }
    }

    return result;
}

// word diff
// ==========

///
pub fn get_word_diff<'a>(color: bool, old: &str, new: &str) -> Vec<Spans<'a>> {
    // Create changeset
    let Changeset { diffs, .. } = Changeset::new(old, new, "\n");

    // create result
    let mut result = vec![];

    for i in 0..diffs.len() {
        match diffs[i] {
            // Same line.
            Difference::Same(ref diff_data_x) => {
                for line in diff_data_x.lines() {
                    let line_data = Spans::from(format!("   {}\n", line));
                    result.push(line_data);
                }
            }

            // Add line.
            Difference::Add(ref diff_data_x) => {
                // line Spans
                let mut line_data = vec![];

                // check lines.
                if i > 0 {
                    match diffs[i - 1] {
                        // Remvoe positon.
                        Difference::Rem(ref diff_data_y) => {
                            let Changeset { diffs, .. } =
                                Changeset::new(diff_data_y, diff_data_x, " ");
                            for c2 in diffs {
                                match c2 {
                                    // Same
                                    Difference::Same(ref char) => {
                                        for l in char.split("\n") {
                                            //
                                            line_data.push(Span::styled(
                                                l.to_string().clone(),
                                                Style::default().fg(Color::Green),
                                            ));
                                            line_data.push(Span::styled(
                                                " ",
                                                Style::default().fg(Color::Green),
                                            ))
                                        }
                                    }

                                    // Add
                                    Difference::Add(ref char) => {
                                        for l in char.split("\n") {
                                            //
                                            line_data.push(Span::styled(
                                                l.to_string().clone(),
                                                Style::default().fg(Color::White).bg(Color::Green),
                                            ));
                                            line_data.push(Span::styled(
                                                " ",
                                                Style::default().fg(Color::Green),
                                            ))
                                        }
                                    }

                                    // No data.
                                    _ => {}
                                }
                            }
                        }
                        //
                        _ => {
                            for l in diff_data_x.split("\n") {
                                //
                                line_data.push(Span::styled(
                                    l.to_string().clone(),
                                    Style::default().fg(Color::Red),
                                ));
                                line_data.push(Span::styled(" ", Style::default().fg(Color::Red)))
                            }
                        }
                    }
                }

                result.push(Spans::from(line_data));
            }

            // Add line.
            Difference::Rem(ref diff_data_x) => {
                // line Spans
                let mut line_data = vec![];
                // line_data.push(Span::styled("-  ", Style::default().fg(Color::Red)));

                if i > 0 {
                    match diffs[i - 1] {
                        // Remvoe positon.
                        Difference::Add(ref diff_data_y) => {
                            let Changeset { diffs, .. } =
                                Changeset::new(diff_data_y, diff_data_x, " ");
                            for c in diffs {
                                match c {
                                    // Same
                                    Difference::Same(ref char) => {
                                        //
                                        line_data.push(Span::styled(
                                            char.clone(),
                                            Style::default().fg(Color::Red),
                                        ))
                                    }

                                    // Rem
                                    Difference::Rem(ref char) => {
                                        //
                                        line_data.push(Span::styled(
                                            char.clone(),
                                            Style::default().fg(Color::White).bg(Color::Red),
                                        ))
                                    }

                                    // No data.
                                    _ => {}
                                }
                            }
                        }

                        //
                        _ => {
                            //
                            line_data.push(Span::styled(
                                format!("-  {}", diff_data_x.clone()),
                                Style::default().fg(Color::Red),
                            ))
                        }
                    }
                }

                result.push(Spans::from(line_data));
            }
        }
    }

    return result;
}
