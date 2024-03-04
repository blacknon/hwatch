// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// modules
use ansi_parser::{AnsiParser, AnsiSequence, Output};
use difference::{Changeset, Difference};
use heapless::consts::*;
use regex::Regex;
use std::borrow::Cow;
use std::cmp;
use std::fmt::Write;
use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    prelude::Line,
};

// local const
use crate::ansi;
use crate::LINE_ENDING;

pub trait StringExt {
    fn expand_tabs(&self, tab_size: u16) -> Cow<str>;
}

impl<T> StringExt for T
where
    T: AsRef<str>,
{
    fn expand_tabs(&self, tab_size: u16) -> Cow<str> {
        let s = self.as_ref();
        let tab = '\t';

        if s.contains(tab) {
            let mut res = String::new();
            let mut last_pos = 0;

            while let Some(pos) = &s[last_pos..].find(tab) {
                res.push_str(&s[last_pos..*pos + last_pos]);

                let spaces_to_add = if tab_size != 0 {
                    tab_size - (*pos as u16 % tab_size)
                } else {
                    0
                };

                if spaces_to_add != 0 {
                    let _ = write!(res, "{:width$}", "", width = spaces_to_add as usize);
                }

                last_pos += *pos + 1;
            }

            res.push_str(&s[last_pos..]);

            Cow::from(res)
        } else {
            Cow::from(s)
        }
    }
}

///
fn expand_line_tab(data: &str, tab_size: u16) -> String {
    let mut result_vec: Vec<String> = vec![];
    for d in data.lines() {
        let l = d.expand_tabs(tab_size).to_string();
        result_vec.push(l);
    }

    result_vec.join("\n")
}

// plane output
// ==========
///
pub fn get_plane_output<'a>(
    color: bool,
    line_number: bool,
    base_text: &str,
    is_filter: bool,
    is_regex_filter: bool,
    filtered_text: &str,
    tab_size: u16,
) -> Vec<Line<'a>> {
    let text = &expand_line_tab(base_text, tab_size);

    // set result `output_data`.
    let mut output_data = vec![];

    // set filter regex.
    let mut pattern: Regex = Regex::new(filtered_text).unwrap();
    if !color && is_filter {
        pattern = Regex::new(&regex::escape(filtered_text)).unwrap();

        if is_regex_filter {
            pattern = Regex::new(filtered_text).unwrap();
        }
    }

    if is_filter && !color {
        // line_span is vec for span on a line-by-line basis
        let mut line_span = vec![];

        // line number
        let mut counter = 1;
        let header_width = &text.split('\n').clone().count().to_string().chars().count();

        let mut last_match: usize = 0;

        for mch in pattern.find_iter(text) {
            let start: usize = mch.start();
            let end: usize = mch.end();

            // before regex hit.
            let before_range_text = &text[last_match..start];

            // regex hit.
            let range_text = &text[start..end];

            // split newline to Spans, at before_range_text
            for (before_range_count, before_text_line) in before_range_text.split('\n').enumerate()
            {
                if before_range_count > 0 {
                    let line_data = line_span.clone();
                    output_data.push(Line::from(line_data));
                    line_span = vec![];
                    counter += 1;
                }

                if line_number && line_span.is_empty() {
                    line_span.push(Span::styled(
                        format!("{counter:>header_width$} | "),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                // push to line_span at before_text_line.
                line_span.push(Span::from(before_text_line.to_string()));
            }

            // split newline to Spans, at range_text
            for (range_count, text_line) in range_text.split('\n').enumerate() {
                if range_count > 0 {
                    if line_number && line_span.is_empty() {
                        line_span.push(Span::styled(
                            format!("{counter:>header_width$} | "),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    let line_data = line_span.clone();
                    output_data.push(Line::from(line_data));
                    line_span = vec![];
                    counter += 1;
                }

                // push to line_span at text_line.
                line_span.push(Span::styled(
                    text_line.to_string(),
                    Style::default().add_modifier(Modifier::REVERSED),
                ));
            }

            last_match = end;
        }

        // last push
        let last_str = &text[last_match..];

        for last_str_line in last_str.split('\n') {
            let mut last_str_line_span = vec![];
            if !line_span.is_empty() {
                last_str_line_span = line_span;
                line_span = vec![];
            }

            if line_number && last_str_line_span.is_empty() {
                last_str_line_span.push(Span::styled(
                    format!("{counter:>header_width$} | "),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            last_str_line_span.push(Span::from(String::from(last_str_line)));

            output_data.push(Line::from(last_str_line_span));

            counter += 1;
        }
    } else {
        let lines = text.split('\n');

        let mut counter = 1;
        let header_width = lines.clone().count().to_string().chars().count();

        for l in lines {
            let mut line_span = vec![];

            if line_number {
                line_span.push(Span::styled(
                    format!("{counter:>header_width$} | "),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if color {
                let data = ansi::bytes_to_text(format!("{l}\n").as_bytes());

                for d in data.lines {
                    line_span.extend(d.spans);
                }
            } else {
                line_span.push(Span::from(String::from(l)));
            }

            output_data.push(Line::from(line_span));
            counter += 1;
        }
    }

    output_data
}

// watch diff
// ==========

///
pub fn get_watch_diff<'a>(
    color: bool,
    line_number: bool,
    old: &str,
    new: &str,
    tab_size: u16,
) -> Vec<Line<'a>> {
    //
    let old_text = &expand_line_tab(old, tab_size);
    let new_text = &expand_line_tab(new, tab_size);

    let mut result = vec![];

    // output to vector
    let mut old_vec: Vec<&str> = old_text.lines().collect();
    let mut new_vec: Vec<&str> = new_text.lines().collect();

    // get max line
    let max_line = cmp::max(old_vec.len(), new_vec.len());

    let mut counter = 1;
    let header_width = max_line.to_string().chars().count();

    // for diff lines
    for i in 0..max_line {
        // push empty line
        if old_vec.len() <= i {
            old_vec.push("");
        }
        if new_vec.len() <= i {
            new_vec.push("");
        }

        let old_line = old_vec[i];
        let new_line = new_vec[i];

        let mut line_data = match color {
            false => get_watch_diff_line(&old_line, &new_line),
            true => get_watch_diff_line_with_ansi(&old_line, &new_line),
        };

        if line_number {
            line_data.spans.insert(
                0,
                Span::styled(
                    format!("{counter:>header_width$} | "),
                    Style::default().fg(Color::DarkGray),
                ),
            );
        }

        result.push(line_data);

        counter += 1;
    }

    result
}

///
fn get_watch_diff_line<'a>(old_line: &str, new_line: &str) -> Line<'a> {
    // If the contents are the same line.
    if old_line == new_line {
        return Line::from(String::from(new_line));
    }

    // Decompose lines by character.
    let mut old_line_chars: Vec<char> = old_line.chars().collect();
    let mut new_line_chars: Vec<char> = new_line.chars().collect();

    // 00a0 ... non-breaking space.
    // NOTE: Used because tui-rs skips regular space characters.
    let space: char = '\u{00a0}';
    let max_char = cmp::max(old_line_chars.len(), new_line_chars.len());

    let mut _result = vec![];
    for x in 0..max_char {
        if old_line_chars.len() <= x {
            old_line_chars.push(space);
        }

        if new_line_chars.len() <= x {
            new_line_chars.push(space);
        }

        let old_char = old_line_chars[x];
        let new_char = new_line_chars[x];

        if old_char != new_char {
            let mut data: Span;
            if new_char == space {
                data = Span::from(' '.to_string());
                data.style = Style::default().add_modifier(Modifier::REVERSED);
            } else {
                data = Span::styled(
                    new_line_chars[x].to_string(),
                    Style::default().add_modifier(Modifier::REVERSED),
                );
            }
            // add span
            _result.push(data);
        } else {
            // add span
            _result.push(Span::styled(
                new_line_chars[x].to_string(),
                Style::default(),
            ));
        }
    }

    // last char
    // NOTE: NBSP used as tui-rs trims regular spaces.
    _result.push(Span::styled(space.to_string(), Style::default()));

    Line::from(_result)
}

///
fn get_watch_diff_line_with_ansi<'a>(old_line: &str, new_line: &str) -> Line<'a> {
    // If the contents are the same line.
    if old_line == new_line {
        let new_spans = ansi::bytes_to_text(format!("{new_line}\n").as_bytes());
        if let Some(spans) = new_spans.into_iter().next() {
            return spans;
        }
    }

    let old_colored_spans = gen_ansi_all_set_str(old_line);
    let new_colored_spans = gen_ansi_all_set_str(new_line);
    let mut old_spans = vec![];
    for mut old_span in old_colored_spans {
        old_spans.append(&mut old_span);
        // break;
    }
    let mut new_spans = vec![];
    for mut new_span in new_colored_spans {
        new_spans.append(&mut new_span);
        // break;
    }

    // 00a0 ... non-breaking space.
    // NOTE: Used because tui-rs skips regular space characters.
    let space = '\u{00a0}'.to_string();
    let max_span = cmp::max(old_spans.len(), new_spans.len());
    //
    let mut _result = vec![];
    for x in 0..max_span {
        //
        if old_spans.len() <= x {
            old_spans.push(Span::from(space.to_string()));
        }

        //
        if new_spans.len() <= x {
            new_spans.push(Span::from(space.to_string()));
        }

        //
        if old_spans[x].content != new_spans[x].content || old_spans[x].style != new_spans[x].style
        {
            if new_spans[x].content == space {
                let mut data = Span::from(' '.to_string());
                data.style = Style::default().add_modifier(Modifier::REVERSED);
                new_spans[x] = data;
            } else {
                // add span
                new_spans[x].style = new_spans[x]
                    .style
                    .patch(Style::default().add_modifier(Modifier::REVERSED));
            }
        }

        //
        _result.push(new_spans[x].clone());
    }

    // last char
    // NOTE: Added hidden characters as tui-rs forces trimming of end-of-line spaces.
    _result.push(Span::styled(space, Style::default()));

    //
    Line::from(_result)
}

// line diff
// ==========

///
pub fn get_line_diff<'a>(
    color: bool,
    line_number: bool,
    is_only_diffline: bool,
    old: &str,
    new: &str,
    tab_size: u16,
) -> Vec<Line<'a>> {
    let old_text = &expand_line_tab(old, tab_size);
    let new_text = &expand_line_tab(new, tab_size);

    // Create changeset
    let Changeset { diffs, .. } = Changeset::new(&old_text, &new_text, LINE_ENDING);

    // old and new text's line count.
    let old_len = &old_text.lines().count();
    let new_len = &new_text.lines().count();

    // get line_number width
    let header_width = cmp::max(old_len, new_len).to_string().chars().count();

    // line_number counter
    let mut old_counter = 1;
    let mut new_counter = 1;

    // create result
    let mut result = vec![];

    (0..diffs.len()).for_each(|i| {
        match diffs[i] {
            // Same line.
            Difference::Same(ref diff_data) => {
                for l in diff_data.lines() {
                    let line = l.expand_tabs(tab_size);
                    let mut data = if color {
                        // ansi color code => rs-tui colored span.
                        let mut colored_span = vec![Span::from("   ")];
                        let colored_data = ansi::bytes_to_text(format!("{line}\n").as_bytes());
                        for d in colored_data.lines {
                            for x in d.spans {
                                colored_span.push(x);
                            }
                        }
                        Line::from(colored_span)
                    } else {
                        // to string => rs-tui span.
                        Line::from(format!("   {line}\n"))
                    };

                    if line_number {
                        data.spans.insert(
                            0,
                            Span::styled(
                                format!("{new_counter:>header_width$} | "),
                                Style::default().fg(Color::DarkGray),
                            ),
                        );
                    }

                    if !is_only_diffline {
                        result.push(data);
                    }

                    // add counter
                    old_counter += 1;
                    new_counter += 1;
                }
            }

            // Add line.
            Difference::Add(ref diff_data) => {
                for l in diff_data.lines() {
                    let line = l.expand_tabs(tab_size);
                    let mut data = if color {
                        // ansi color code => parse and delete. to rs-tui span(green).
                        let strip_str = get_ansi_strip_str(&line);
                        Line::from(Span::styled(
                            format!("+  {strip_str}\n"),
                            Style::default().fg(Color::Green),
                        ))
                    } else {
                        // to string => rs-tui span.
                        Line::from(Span::styled(
                            format!("+  {line}\n"),
                            Style::default().fg(Color::Green),
                        ))
                    };

                    if line_number {
                        data.spans.insert(
                            0,
                            Span::styled(
                                format!("{new_counter:>header_width$} | "),
                                Style::default().fg(Color::Rgb(56, 119, 120)),
                            ),
                        );
                    }

                    result.push(data);

                    // add new_counter
                    new_counter += 1;
                }
            }

            // Remove line.
            Difference::Rem(ref diff_data) => {
                for l in diff_data.lines() {
                    let line = l.expand_tabs(tab_size);
                    let mut data = if color {
                        // ansi color code => parse and delete. to rs-tui span(green).
                        let strip_str = get_ansi_strip_str(&line);
                        Line::from(Span::styled(
                            format!("-  {strip_str}\n"),
                            Style::default().fg(Color::Red),
                        ))
                    } else {
                        // to string => rs-tui span.
                        Line::from(Span::styled(
                            format!("-  {line}\n"),
                            Style::default().fg(Color::Red),
                        ))
                    };

                    if line_number {
                        data.spans.insert(
                            0,
                            Span::styled(
                                format!("{old_counter:>header_width$} | "),
                                Style::default().fg(Color::Rgb(118, 0, 0)),
                            ),
                        );
                    }

                    result.push(data);

                    // add old_counter
                    old_counter += 1;
                }
            }
        }
    });

    result
}

// word diff
// ==========

///
pub fn get_word_diff<'a>(
    color: bool,
    line_number: bool,
    is_only_diffline: bool,
    old: &str,
    new: &str,
    tab_size: u16,
) -> Vec<Line<'a>> {
    let old_text = &expand_line_tab(old, tab_size);
    let new_text = &expand_line_tab(new, tab_size);

    // Create changeset
    let Changeset { diffs, .. } = Changeset::new(&old_text, &new_text, LINE_ENDING);

    // old and new text's line count.
    let old_len = &old_text.lines().count();
    let new_len = &new_text.lines().count();

    // get line_number width
    let header_width = cmp::max(old_len, new_len).to_string().chars().count();

    // line_number counter
    let mut old_counter = 1;
    let mut new_counter = 1;

    // create result
    let mut result = vec![];

    for i in 0..diffs.len() {
        match diffs[i] {
            // Same line.
            Difference::Same(ref diff_data) => {
                for l in diff_data.lines() {
                    let line = l.expand_tabs(tab_size);
                    let mut data = if color {
                        // ansi color code => rs-tui colored span.
                        let mut colored_span = vec![Span::from("   ")];
                        let colored_data = ansi::bytes_to_text(format!("{line}\n").as_bytes());
                        for d in colored_data.lines {
                            for x in d.spans {
                                colored_span.push(x);
                            }
                        }
                        Line::from(colored_span)
                    } else {
                        // to string => rs-tui span.
                        Line::from(format!("   {line}\n"))
                    };

                    if line_number {
                        data.spans.insert(
                            0,
                            Span::styled(
                                format!("{new_counter:>header_width$} | "),
                                Style::default().fg(Color::DarkGray),
                            ),
                        );
                    }

                    if !is_only_diffline {
                        result.push(data);
                    }

                    // add counter
                    old_counter += 1;
                    new_counter += 1;
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

                    lines_data = get_word_diff_addline(color, before_diffs, diff_data.to_string())
                } else {
                    for l in diff_data.lines() {
                        let line = l.expand_tabs(tab_size);
                        let data = if color {
                            get_ansi_strip_str(&line)
                        } else {
                            line.to_string()
                        };
                        lines_data.push(vec![Span::styled(
                            data.to_string(),
                            Style::default().fg(Color::Green),
                        )]);
                    }
                }

                for line_data in lines_data {
                    let mut data = vec![Span::styled("+  ", Style::default().fg(Color::Green))];
                    for line in line_data {
                        data.push(line);
                    }

                    if line_number {
                        data.insert(
                            0,
                            Span::styled(
                                format!("{new_counter:>header_width$} | "),
                                Style::default().fg(Color::Rgb(56, 119, 120)),
                            ),
                        );
                    }

                    result.push(Line::from(data.clone()));

                    // add new_counter
                    new_counter += 1;
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
                if diffs.len() > i + 1 {
                    let after_diffs: &Difference = &diffs[i + 1];

                    lines_data = get_word_diff_remline(color, after_diffs, diff_data.to_string())
                } else {
                    for line in diff_data.lines() {
                        let data = if color {
                            get_ansi_strip_str(line)
                        } else {
                            line.to_string()
                        };
                        lines_data.push(vec![Span::styled(
                            data.to_string(),
                            Style::default().fg(Color::Red),
                        )]);
                    }
                }

                for line_data in lines_data {
                    let mut data = vec![Span::styled("-  ", Style::default().fg(Color::Red))];
                    for line in line_data {
                        data.push(line);
                    }

                    if line_number {
                        data.insert(
                            0,
                            Span::styled(
                                format!("{old_counter:>header_width$} | "),
                                Style::default().fg(Color::Rgb(118, 0, 0)),
                            ),
                        );
                    }

                    result.push(Line::from(data.clone()));

                    // add old_counter
                    old_counter += 1;
                }
            }
        }
    }

    result
}

/// This Function when there is an additional line in word_diff and there is a previous diff.
///
fn get_word_diff_addline<'a>(
    color: bool,
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
        Difference::Rem(before_diff_data) => {
            // Craete Changeset at `Addlind` and `Before Diff Data`.
            let Changeset { diffs, .. } = Changeset::new(before_diff_data, &diff_data, " ");

            //
            for c in diffs {
                match c {
                    // Same
                    Difference::Same(ref char) => {
                        let same_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::Green),
                            char,
                        );

                        for (counter, lines) in same_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // Add
                    Difference::Add(ref char) => {
                        let add_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::White).bg(Color::Green),
                            char,
                        );

                        for (counter, lines) in add_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // No data.
                    _ => {}
                }
            }
        }

        // Add line
        _ => {
            for line in diff_data.lines() {
                let data = if color {
                    get_ansi_strip_str(line)
                } else {
                    line.to_string()
                };
                let line_data = vec![Span::styled(
                    data.to_string(),
                    Style::default().fg(Color::Green),
                )];
                result.push(line_data);
            }
        }
    }

    if !line_data.is_empty() {
        result.push(line_data);
    }

    result
}

///
fn get_word_diff_remline<'a>(
    color: bool,
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
        Difference::Add(after_diffs_data) => {
            // Craete Changeset at `Addlind` and `Before Diff Data`.
            let Changeset { diffs, .. } = Changeset::new(&diff_data, after_diffs_data, " ");

            //
            for c in diffs {
                match c {
                    // Same
                    Difference::Same(ref char) => {
                        let same_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::Red),
                            char,
                        );

                        for (counter, lines) in same_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // Add
                    Difference::Rem(ref char) => {
                        let add_line = get_word_diff_line_to_spans(
                            color,
                            Style::default().fg(Color::White).bg(Color::Red),
                            char,
                        );

                        for (counter, lines) in add_line.into_iter().enumerate() {
                            if counter > 0 {
                                result.push(line_data);
                                line_data = vec![];
                            }

                            for l in lines {
                                line_data.push(l.clone());
                            }
                        }
                    }

                    // No data.
                    _ => {}
                }
            }
        }

        // Rem line
        _ => {
            for line in diff_data.lines() {
                let data = if color {
                    get_ansi_strip_str(line)
                } else {
                    line.to_string()
                };

                let line_data = vec![Span::styled(
                    data.to_string(),
                    Style::default().fg(Color::Red),
                )];

                result.push(line_data);
            }
        }
    }

    if !line_data.is_empty() {
        result.push(line_data);
    }

    result
}

///
fn get_word_diff_line_to_spans<'a>(
    color: bool,
    style: Style,
    diff_str: &str,
) -> Vec<Vec<Span<'a>>> {
    // result
    let mut result = vec![];

    for l in diff_str.split('\n') {
        let text = if color {
            get_ansi_strip_str(l)
        } else {
            l.to_string()
        };

        let line = vec![
            Span::styled(text.clone(), style),
            Span::styled(" ", Style::default()),
        ];

        result.push(line);
    }

    result
}

// Ansi Color Code parse
// ==========

/// Apply ANSI color code character by character.
fn gen_ansi_all_set_str<'b>(text: &str) -> Vec<Vec<Span<'b>>> {
    // set Result
    let mut result = vec![];

    // ansi reset code heapless_vec
    let mut ansi_reset_vec = heapless::Vec::<u8, U5>::new();
    let _ = ansi_reset_vec.push(0);

    // get ansi reset code string
    let ansi_reset_seq = AnsiSequence::SetGraphicsMode(ansi_reset_vec);
    let ansi_reset_seq_str = ansi_reset_seq.to_string();

    // init sequence.
    let mut sequence: AnsiSequence;
    let mut sequence_str = "".to_string();

    // text processing
    let mut processed_text = vec![];
    for block in text.ansi_parse() {
        match block {
            Output::TextBlock(text) => {
                for char in text.chars() {
                    let append_text = if !sequence_str.is_empty() {
                        format!("{sequence_str}{char}{ansi_reset_seq_str}")
                    } else {
                        format!("{char}")
                    };

                    // parse ansi text to tui text.
                    let data = ansi::bytes_to_text(format!("{append_text}\n").as_bytes());
                    if let Some(d) = data.into_iter().next() {
                        for x in d.spans {
                            processed_text.push(x);
                        }
                    }
                }
            }
            Output::Escape(seq) => {
                sequence = seq;
                sequence_str = sequence.to_string();
            }
        }
    }

    result.push(processed_text);

    result
}

///
fn get_ansi_strip_str(text: &str) -> String {
    let mut line_str = "".to_string();
    for block in text.ansi_parse() {
        if let Output::TextBlock(text) = block {
            line_str.push_str(text);
        }
    }

    line_str
}
