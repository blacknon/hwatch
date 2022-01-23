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
// use termwiz::escape::parser::Parser;
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
    //     // TODO: 書く. 差分発生箇所をANSIで記述して、それをansi4tuiに渡して変換する方式とする
    //     // If the contents are the same line.
    //     if old_line == new_line {
    //         return Spans::from(String::from(new_line));
    //     }

    // let debug_span = Span::styled(" ", Style::default().fg(Color::Yellow));
    // debug_span.style.add_modifier(modifier: Modifier) = Style::default().fg(Color::Yellow);

    //     // Decompose lines by character.
    //     let mut old_line_chars: Vec<char> = old_line..collect();
    //     let mut new_line_chars: Vec<char> = new_line.escape_default().collect();

    //     let space: char = ' ';
    //     let max_char = cmp::max(old_line_chars.len(), new_line_chars.len());

    //     let mut _result = vec![];
    //     // let mut _result_anis = vec![];

    //     for x in 0..max_char {
    //         if old_line_chars.len() <= max_char {
    //             old_line_chars.push(space);
    //         }

    //         if new_line_chars.len() <= max_char {
    //             new_line_chars.push(space);
    //         }

    //         if old_line_chars[x] != new_line_chars[x] {
    //             // add span
    //             _result.push(Span::styled(
    //                 new_line_chars[x].to_string(),
    //                 Style::default().add_modifier(Modifier::REVERSED),
    //             ));
    //         } else {
    //             // add span
    //             _result.push(Span::styled(
    //                 new_line_chars[x].to_string(),
    //                 Style::default(),
    //             ));
    //         }
    //     }

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
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
                    let line_data = Spans::from(format!("   {}\n", line));
                    result.push(line_data);
                }
            }

            // Add line.
            Difference::Add(ref diff_data) => {
                // line Spans.
                // it is lines data <Vec<Vec<Span<'a>>>>
                // ex)
                // [   // 1st line...
                //     [Sapn, Span, Span, ...],
                //     // 2nd line...
                //     [Sapn, Span, Span, ...],
                //     // 3rd line...
                //     [Sapn, Span, Span, ...],
                // ]
                let mut lines_data = vec![];

                // check lines.
                if i > 0 {
                    let before_diffs = &diffs[i - 1];

                    lines_data = get_word_diff_addline(before_diffs, diff_data.to_string())
                } else {
                    for line in diff_data.lines() {
                        lines_data.push(vec![Span::styled(
                            line.to_string(),
                            Style::default().fg(Color::Green),
                        )]);
                    }
                }

                for line_data in lines_data {
                    let mut data = vec![Span::styled("+  ", Style::default().fg(Color::Green))];
                    for line in line_data {
                        data.push(line);
                    }

                    result.push(Spans::from(data.clone()));
                }
            }

            // Remove line.
            Difference::Rem(ref diff_data) => {
                // line Spans.
                // it is lines data <Vec<Vec<Span<'a>>>>
                // ex)
                // [   // 1st line...
                //     [Sapn, Span, Span, ...],
                //     // 2nd line...
                //     [Sapn, Span, Span, ...],
                //     // 3rd line...
                //     [Sapn, Span, Span, ...],
                // ]
                let mut lines_data = vec![];

                // check lines.
                if i > 0 {
                    let after_diffs = &diffs[i + 1];

                    lines_data = get_word_diff_remline(after_diffs, diff_data.to_string())
                } else {
                    for line in diff_data.lines() {
                        lines_data.push(vec![Span::styled(
                            line.to_string(),
                            Style::default().fg(Color::Red),
                        )]);
                    }
                }

                for line_data in lines_data {
                    let mut data = vec![Span::styled("-  ", Style::default().fg(Color::Red))];
                    for line in line_data {
                        data.push(line);
                    }

                    result.push(Spans::from(data.clone()));
                }
            }
        }
    }

    return result;
}

/// This Function when there is an additional line in word_diff and there is a previous diff.
///
fn get_word_diff_addline<'a>(
    before_diffs: &difference::Difference,
    diff_data: String,
) -> Vec<Vec<Span<'a>>> {
    // result is Vec<Vec<Span>>
    // ex)
    // [   // 1st line...
    //     [Sapn, Span, Span, ...],
    //     // 2nd line...
    //     [Sapn, Span, Span, ...],
    //     // 3rd line...
    //     [Sapn, Span, Span, ...],
    // ]
    let mut result = vec![];

    // line_data is Vec<Span>
    // ex) [Span, Span, Span, ...]
    let mut line_data = vec![];

    match before_diffs {
        // Change Line.
        &Difference::Rem(ref before_diff_data) => {
            // Craete Changeset at `Addlind` and `Before Diff Data`.
            let Changeset { diffs, .. } = Changeset::new(before_diff_data, &diff_data, " ");

            //
            for c in diffs {
                match c {
                    // Same
                    Difference::Same(ref char) => {
                        let same_line =
                            get_word_diff_line_to_spans(Style::default().fg(Color::Green), char);
                        let mut counter = 0;

                        for lines in same_line {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }

                            counter += 1;
                        }
                    }

                    // Add
                    Difference::Add(ref char) => {
                        let add_line = get_word_diff_line_to_spans(
                            Style::default().fg(Color::White).bg(Color::Green),
                            char,
                        );
                        let mut counter = 0;

                        for lines in add_line {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }

                            counter += 1;
                        }
                    }

                    // No data.
                    _ => {}
                }
            }
        }

        // Add Newline
        _ => {
            for line in diff_data.lines() {
                let line_data = vec![Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Green),
                )];
                result.push(line_data);
            }
        }
    }

    if line_data.len() > 0 {
        result.push(line_data);
        line_data = vec![];
    }

    return result;
}

///
fn get_word_diff_remline<'a>(
    after_diffs: &difference::Difference,
    diff_data: String,
) -> Vec<Vec<Span<'a>>> {
    // result is Vec<Vec<Span>>
    // ex)
    // [   // 1st line...
    //     [Sapn, Span, Span, ...],
    //     // 2nd line...
    //     [Sapn, Span, Span, ...],
    //     // 3rd line...
    //     [Sapn, Span, Span, ...],
    // ]
    let mut result = vec![];

    // line_data is Vec<Span>
    // ex) [Span, Span, Span, ...]
    let mut line_data = vec![];

    match after_diffs {
        // Change Line.
        &Difference::Add(ref after_diffs_data) => {
            // Craete Changeset at `Addlind` and `Before Diff Data`.
            let Changeset { diffs, .. } = Changeset::new(&diff_data, after_diffs_data, " ");

            //
            for c in diffs {
                match c {
                    // Same
                    Difference::Same(ref char) => {
                        let same_line =
                            get_word_diff_line_to_spans(Style::default().fg(Color::Red), char);
                        let mut counter = 0;

                        for lines in same_line {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }

                            counter += 1;
                        }
                    }

                    // Add
                    Difference::Rem(ref char) => {
                        let add_line = get_word_diff_line_to_spans(
                            Style::default().fg(Color::White).bg(Color::Red),
                            char,
                        );
                        let mut counter = 0;

                        for lines in add_line {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }

                            counter += 1;
                        }
                    }

                    // No data.
                    _ => {}
                }
            }
        }

        // Add Newline
        _ => {
            for line in diff_data.lines() {
                let line_data = vec![Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Red),
                )];
                result.push(line_data);
            }
        }
    }

    if line_data.len() > 0 {
        result.push(line_data);
        line_data = vec![];
    }

    return result;
}

///
fn get_word_diff_line_to_spans<'a>(style: Style, diff_str: &str) -> Vec<Vec<Span<'a>>> {
    // result
    let mut result = vec![];
    let mut counter = 0;

    // Decompose a string for each character.
    let chars: Vec<char> = diff_str.chars().collect();

    for l in diff_str.split("\n") {
        let mut line = vec![];

        line.push(Span::styled(l.to_string().clone(), style));
        line.push(Span::styled(" ", Style::default()));
        result.push(line);

        counter += 1;
    }

    return result;
}
