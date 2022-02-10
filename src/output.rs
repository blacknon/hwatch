// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// modules
use ansi_parser::{AnsiParser, AnsiSequence, Output};
use difference::{Changeset, Difference};
use heapless::consts::*;
use regex::Regex;
use std::cmp;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

// plane output
// ==========

///
pub fn get_plane_output<'a>(
    color: bool,
    line_number: bool,
    text: &str,
    is_filter: bool,
    is_regex_filter: bool,
    filtered_text: &str,
) -> Vec<Spans<'a>> {
    // set result `output_data`.
    let mut output_data = vec![];

    // set filter regex.
    let mut pattern: Regex = Regex::new(&filtered_text).unwrap();
    if !color && is_filter {
        pattern = Regex::new(&regex::escape(&filtered_text)).unwrap();

        if is_regex_filter {
            pattern = Regex::new(&filtered_text).unwrap();
        }
    }

    if is_filter {
        // line_span is vec for span on a line-by-line basis
        let mut line_span = vec![];

        // line number
        let mut counter = 1;
        let header_width = &text
            .split("\n")
            .clone()
            .collect::<Vec<&str>>()
            .len()
            .to_string()
            .chars()
            .collect::<Vec<char>>()
            .len();

        let mut last_match: usize = 0;

        for mch in pattern.find_iter(&text) {
            let start: usize = mch.start();
            let end: usize = mch.end();

            // before regex hit.
            let before_range_text = &text[last_match..start];

            // regex hit.
            let range_text = &text[start..end];

            // split newline to Spans, at before_range_text
            let mut before_range_count = 0;
            for before_text_line in before_range_text.split('\n') {
                if before_range_count > 0 {
                    let line_data = line_span.clone();
                    output_data.push(Spans::from(line_data));
                    line_span = vec![];
                    counter += 1;
                }

                if line_number && line_span.len() == 0 {
                    line_span.push(Span::styled(
                        format!("{:>wid$} | ", counter, wid = header_width),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                // push to line_span at before_text_line.
                line_span.push(Span::from(before_text_line.to_string()));

                before_range_count += 1;
            }

            // split newline to Spans, at range_text
            let mut range_count = 0;
            for text_line in range_text.split('\n') {
                if range_count > 0 {
                    if line_number && line_span.len() == 0 {
                        line_span.push(Span::styled(
                            format!("{:>wid$} | ", counter, wid = header_width),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    let line_data = line_span.clone();
                    output_data.push(Spans::from(line_data));
                    line_span = vec![];
                    counter += 1;
                }

                // push to line_span at text_line.
                line_span.push(Span::styled(
                    text_line.to_string(),
                    Style::default().add_modifier(Modifier::REVERSED),
                ));

                range_count += 1;
            }

            last_match = end;
        }

        // last push
        let last_str = &text[last_match..];

        for last_str_line in last_str.split('\n') {
            let mut last_str_line_span = vec![];
            if line_span.len() > 0 {
                last_str_line_span = line_span;
                line_span = vec![];
            }

            if line_number && last_str_line_span.len() == 0 {
                last_str_line_span.push(Span::styled(
                    format!("{:>wid$} | ", counter, wid = header_width),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            last_str_line_span.push(Span::from(String::from(last_str_line)));

            output_data.push(Spans::from(last_str_line_span));

            counter += 1;
        }
    } else {
        let lines = text.split("\n");

        let mut counter = 1;
        let header_width = lines
            .clone()
            .collect::<Vec<&str>>()
            .len()
            .to_string()
            .chars()
            .collect::<Vec<char>>()
            .len();

        for l in lines {
            let mut line_span = vec![];

            if line_number {
                line_span.push(Span::styled(
                    format!("{:>wid$} | ", counter, wid = header_width),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if color {
                let data = ansi4tui::bytes_to_text(format!("{}\n", l).as_bytes().to_vec());

                for d in data.lines {
                    line_span.extend(d.0);
                }
            } else {
                line_span.push(Span::from(String::from(l)));
            }

            output_data.push(Spans::from(line_span));
            counter += 1;
        }
    }

    return output_data;
}

// watch diff
// ==========

///
pub fn get_watch_diff<'a>(color: bool, line_number: bool, old: &str, new: &str) -> Vec<Spans<'a>> {
    let mut result = vec![];

    // output to vector
    let mut old_vec: Vec<&str> = old.lines().collect();
    let mut new_vec: Vec<&str> = new.lines().collect();

    // get max line
    let max_line = cmp::max(old_vec.len(), new_vec.len());

    let mut counter = 1;
    let header_width = max_line.to_string().chars().collect::<Vec<char>>().len();

    // for diff lines
    for i in 0..max_line {
        // push empty line
        if old_vec.len() <= i {
            old_vec.push("");
        }
        if new_vec.len() <= i {
            new_vec.push("");
        }

        let mut line_data: Spans;

        match color {
            false => line_data = get_watch_diff_line(old_vec[i], new_vec[i]),
            true => {
                // line_data = Spans::from(vec![]);
                line_data = get_watch_diff_line_with_ansi(old_vec[i], new_vec[i])
            }
        }

        if line_number {
            line_data.0.insert(
                0,
                Span::styled(
                    format!("{:>wid$} | ", counter, wid = header_width),
                    Style::default().fg(Color::DarkGray),
                ),
            );
        }

        result.push(line_data);

        counter += 1;
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
        if old_line_chars.len() <= x {
            old_line_chars.push(space);
        }

        if new_line_chars.len() <= x {
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
    // If the contents are the same line.
    if old_line == new_line {
        let new_spans = ansi4tui::bytes_to_text(format!("{}\n", new_line).as_bytes().to_vec());
        for spans in new_spans.lines {
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

    let max_span = cmp::max(old_spans.len(), new_spans.len());
    //
    let mut _result = vec![];
    for x in 0..max_span {
        //
        if old_spans.len() <= x {
            old_spans.push(Span::from(" ".to_string()));
        }

        //
        if new_spans.len() <= x {
            new_spans.push(Span::from(" ".to_string()));
        }

        //
        if old_spans[x].content != new_spans[x].content || old_spans[x].style != new_spans[x].style
        {
            // add span
            new_spans[x].style = new_spans[x]
                .style
                .patch(Style::default().add_modifier(Modifier::REVERSED));
        }

        //
        _result.push(new_spans[x].clone());
    }

    //
    return Spans::from(_result);
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
                    let data: Spans;

                    if color {
                        // ansi color code => rs-tui colored span.
                        let mut colored_span = vec![];
                        colored_span.push(Span::from("   "));
                        let colored_data =
                            ansi4tui::bytes_to_text(format!("{}\n", line).as_bytes().to_vec());
                        for d in colored_data.lines {
                            for x in d.0 {
                                colored_span.push(x);
                            }
                        }
                        data = Spans::from(colored_span);
                    } else {
                        // to string => rs-tui span.
                        data = Spans::from(format!("   {}\n", line));
                    }
                    result.push(data);
                }
            }

            // Add line.
            Difference::Add(ref diff_data) => {
                for line in diff_data.lines() {
                    let data: Spans;
                    if color {
                        // ansi color code => parse and delete. to rs-tui span(green).
                        let strip_str = get_ansi_strip_str(line);
                        data = Spans::from(Span::styled(
                            format!("+  {}\n", strip_str),
                            Style::default().fg(Color::Green),
                        ));
                    } else {
                        // to string => rs-tui span.
                        data = Spans::from(Span::styled(
                            format!("+  {}\n", line),
                            Style::default().fg(Color::Green),
                        ));
                    }

                    result.push(data);
                }
            }

            // Remove line.
            Difference::Rem(ref diff_data) => {
                for line in diff_data.lines() {
                    let data: Spans;
                    if color {
                        // ansi color code => parse and delete. to rs-tui span(green).
                        let strip_str = get_ansi_strip_str(line);
                        data = Spans::from(Span::styled(
                            format!("-  {}\n", strip_str),
                            Style::default().fg(Color::Red),
                        ));
                    } else {
                        // to string => rs-tui span.
                        data = Spans::from(Span::styled(
                            format!("-  {}\n", line),
                            Style::default().fg(Color::Red),
                        ));
                    }

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
    let Changeset { diffs, .. } = Changeset::new(old, new, ::LINE_ENDING);

    // create result
    let mut result = vec![];

    for i in 0..diffs.len() {
        match diffs[i] {
            // Same line.
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
                    let data: Spans;

                    if color {
                        // ansi color code => rs-tui colored span.
                        let mut colored_span = vec![];
                        colored_span.push(Span::from("   "));
                        let colored_data =
                            ansi4tui::bytes_to_text(format!("{}\n", line).as_bytes().to_vec());
                        for d in colored_data.lines {
                            for x in d.0 {
                                colored_span.push(x);
                            }
                        }
                        data = Spans::from(colored_span);
                    } else {
                        // to string => rs-tui span.
                        data = Spans::from(format!("   {}\n", line));
                    }
                    result.push(data);
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
                    for line in diff_data.lines() {
                        let data: String;

                        if color {
                            data = get_ansi_strip_str(line);
                        } else {
                            data = line.to_string();
                        }
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

                    lines_data = get_word_diff_remline(color, after_diffs, diff_data.to_string())
                } else {
                    for line in diff_data.lines() {
                        let data: String;

                        if color {
                            data = get_ansi_strip_str(line);
                        } else {
                            data = line.to_string();
                        }
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
        &Difference::Rem(ref before_diff_data) => {
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
                            color,
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

        // Add line
        _ => {
            for line in diff_data.lines() {
                let data: String;

                if color {
                    data = get_ansi_strip_str(line);
                } else {
                    data = line.to_string();
                }
                let line_data = vec![Span::styled(
                    data.to_string(),
                    Style::default().fg(Color::Green),
                )];
                result.push(line_data);
            }
        }
    }

    if line_data.len() > 0 {
        result.push(line_data);
    }

    return result;
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
        &Difference::Add(ref after_diffs_data) => {
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
                            color,
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

        // Rem line
        _ => {
            for line in diff_data.lines() {
                let data: String;

                if color {
                    data = get_ansi_strip_str(line);
                } else {
                    data = line.to_string();
                }
                let line_data = vec![Span::styled(
                    data.to_string(),
                    Style::default().fg(Color::Red),
                )];

                result.push(line_data);
            }
        }
    }

    if line_data.len() > 0 {
        result.push(line_data);
    }

    return result;
}

///
fn get_word_diff_line_to_spans<'a>(
    color: bool,
    style: Style,
    diff_str: &str,
) -> Vec<Vec<Span<'a>>> {
    // result
    let mut result = vec![];

    for l in diff_str.split("\n") {
        let text: String;
        if color {
            text = get_ansi_strip_str(l);
        } else {
            text = l.to_string();
        }

        let mut line = vec![];

        line.push(Span::styled(text.clone(), style));
        line.push(Span::styled(" ", Style::default()));
        result.push(line);
    }

    return result;
}

// Ansi Color Code parse
// ==========

/// Apply ANSI color code character by character.
fn gen_ansi_all_set_str<'a, 'b>(text: &'a str) -> Vec<Vec<Span<'b>>> {
    // set Result
    let mut result = vec![];

    // text parse ansi color code set.
    let parsed: Vec<Output> = text.ansi_parse().collect();

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
    for block in parsed.into_iter() {
        match block {
            Output::TextBlock(text) => {
                for char in text.chars() {
                    let append_text: String;
                    if &sequence_str.len() > &0 {
                        append_text = format!("{}{}{}", sequence_str, char, ansi_reset_seq_str);
                    } else {
                        append_text = format!("{}", char);
                    }

                    // parse ansi text to tui text.
                    let data =
                        ansi4tui::bytes_to_text(format!("{}\n", append_text).as_bytes().to_vec());
                    for d in data {
                        for x in d.0 {
                            processed_text.push(x);
                        }
                        break;
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

    return result;
}

///
fn get_ansi_strip_str<'a>(text: &'a str) -> String {
    let parsed: Vec<Output> = text.ansi_parse().collect();
    let mut line_str = "".to_string();
    for block in parsed.into_iter() {
        match block {
            Output::TextBlock(text) => {
                line_str.push_str(text);
            }
            _ => {}
        }
    }

    return line_str;
}
