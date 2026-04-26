// Copyright (c) 2026 Blacknon.
// This code from https://github.com/blacknon/ansi4tui/blob/master/src/lib.rs

use termwiz::cell::{Blink, Intensity, Underline};
use termwiz::color::ColorSpec;
use termwiz::escape::{
    csi::{Sgr, CSI},
    parser::Parser,
    Action, ControlCode,
};
use tui::prelude::Line;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Text};

use ratatui as tui;

fn apply_sgr(span_style: &mut Style, sgr: Sgr) {
    if let Sgr::Font(_) = sgr {
        return;
    }

    match sgr {
        Sgr::Reset => *span_style = Style::default(),
        Sgr::Intensity(i) => match i {
            Intensity::Bold => {
                *span_style = span_style.remove_modifier(Modifier::DIM);
                *span_style = span_style.add_modifier(Modifier::BOLD);
            }
            Intensity::Half => {
                *span_style = span_style.add_modifier(Modifier::DIM);
                *span_style = span_style.remove_modifier(Modifier::BOLD);
            }
            Intensity::Normal => {
                *span_style = span_style.remove_modifier(Modifier::DIM);
                *span_style = span_style.remove_modifier(Modifier::BOLD);
            }
        },
        Sgr::Underline(u) => match u {
            Underline::Double | Underline::Single => {
                *span_style = span_style.add_modifier(Modifier::UNDERLINED);
            }
            _ => *span_style = span_style.remove_modifier(Modifier::UNDERLINED),
        },
        Sgr::Blink(b) => match b {
            Blink::Slow => {
                *span_style = span_style.add_modifier(Modifier::SLOW_BLINK);
                *span_style = span_style.remove_modifier(Modifier::RAPID_BLINK);
            }
            Blink::Rapid => {
                *span_style = span_style.remove_modifier(Modifier::SLOW_BLINK);
                *span_style = span_style.add_modifier(Modifier::RAPID_BLINK);
            }
            Blink::None => {
                *span_style = span_style.remove_modifier(Modifier::SLOW_BLINK);
                *span_style = span_style.remove_modifier(Modifier::RAPID_BLINK);
            }
        },
        Sgr::Italic(true) => *span_style = span_style.add_modifier(Modifier::ITALIC),
        Sgr::Italic(false) => *span_style = span_style.remove_modifier(Modifier::ITALIC),
        Sgr::Inverse(true) => *span_style = span_style.add_modifier(Modifier::REVERSED),
        Sgr::Inverse(false) => *span_style = span_style.remove_modifier(Modifier::REVERSED),
        Sgr::Invisible(true) => *span_style = span_style.add_modifier(Modifier::HIDDEN),
        Sgr::Invisible(false) => *span_style = span_style.remove_modifier(Modifier::HIDDEN),
        Sgr::StrikeThrough(true) => *span_style = span_style.add_modifier(Modifier::CROSSED_OUT),
        Sgr::StrikeThrough(false) => {
            *span_style = span_style.remove_modifier(Modifier::CROSSED_OUT)
        }
        Sgr::Foreground(c) => match c {
            ColorSpec::Default => *span_style = span_style.fg(Color::Reset),
            ColorSpec::PaletteIndex(i) => *span_style = span_style.fg(Color::Indexed(i)),
            ColorSpec::TrueColor(rgb) => {
                let rgb_tuple = rgb.to_srgb_u8();
                *span_style = span_style.fg(Color::Rgb(rgb_tuple.0, rgb_tuple.1, rgb_tuple.2));
            }
        },
        Sgr::Background(c) => match c {
            ColorSpec::Default => *span_style = span_style.bg(Color::Reset),
            ColorSpec::PaletteIndex(i) => *span_style = span_style.bg(Color::Indexed(i)),
            ColorSpec::TrueColor(rgb) => {
                let rgb_tuple = rgb.to_srgb_u8();
                *span_style = span_style.bg(Color::Rgb(rgb_tuple.0, rgb_tuple.1, rgb_tuple.2));
            }
        },
        _ => {}
    }
}

/// Converts ANSI-escaped strings to tui-rs compatible text
pub fn bytes_to_text<'a, B: AsRef<[u8]>>(bytes: B) -> Text<'a> {
    let mut parser = Parser::new();
    let parsed = parser.parse_as_vec(bytes.as_ref());

    // each span will be a line
    let mut spans = Vec::<Line>::new();

    // create span buffer
    let mut span_style = Style::default();
    let mut span_text = String::new();

    // list of spans that makes up a Spans
    let mut current_line = Vec::<Span>::new();

    for item in parsed {
        match item {
            // Somehow pass in terminal size here for string buffering, and handle linefeeds and carriage returns
            // separately rather than assuming linefeed includes cr?
            Action::Print(c) => {
                span_text.push(c);
            }
            Action::PrintString(s) => {
                for c in s.chars() {
                    span_text.push(c);
                }
            }
            Action::Control(ControlCode::LineFeed) => {
                // finish the current span
                current_line.push(Span::styled(span_text, span_style));
                span_text = String::new();

                // finish the current line
                spans.push(Line::from(current_line));
                current_line = Vec::new();
            }
            Action::CSI(CSI::Sgr(sgr)) => {
                // finish the current span
                current_line.push(Span::styled(span_text, span_style));
                span_text = String::new();

                apply_sgr(&mut span_style, sgr);
            }
            _ => {}
        }
    }

    if !span_text.is_empty() {
        // finish the current span
        current_line.push(Span::styled(span_text, span_style));
    }

    // push any remaining data
    if !current_line.is_empty() {
        // finish the current line
        spans.push(Line::from(current_line));
    }

    spans.into()
}

// Ansi Color Code parse
// ==========

/// Apply ANSI color code character by character.
pub fn gen_ansi_all_set_str<'b>(text: &str) -> Vec<Vec<Span<'b>>> {
    let mut parser = Parser::new();
    let parsed = parser.parse_as_vec(text.as_bytes());
    let mut result = vec![Vec::new()];
    let mut span_style = Style::default();

    for item in parsed {
        match item {
            Action::Print(c) => result
                .last_mut()
                .unwrap()
                .push(Span::styled(c.to_string(), span_style)),
            Action::PrintString(s) => {
                for c in s.chars() {
                    result
                        .last_mut()
                        .unwrap()
                        .push(Span::styled(c.to_string(), span_style));
                }
            }
            Action::Control(ControlCode::LineFeed) => result.push(Vec::new()),
            Action::CSI(CSI::Sgr(sgr)) => apply_sgr(&mut span_style, sgr),
            _ => {}
        }
    }

    result
}

pub fn get_ansi_strip_str(text: &str) -> String {
    let mut parser = Parser::new();
    let parsed = parser.parse_as_vec(text.as_bytes());
    let mut line_str = String::new();

    for item in parsed {
        match item {
            Action::Print(c) => line_str.push(c),
            Action::PrintString(s) => line_str.push_str(&s),
            Action::Control(ControlCode::LineFeed) => line_str.push('\n'),
            _ => {}
        }
    }

    line_str
}

pub fn escape_ansi(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some('[') = chars.peek() {
                result.push_str("\\x1b[");
                chars.next(); // Consume '['
                while let Some(ch) = chars.peek() {
                    result.push(*ch);
                    if ch.is_alphabetic() {
                        chars.next(); // Consume the letter
                        break;
                    }
                    chars.next(); // Consume the number or ';'
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui::style::{Color, Modifier};

    #[test]
    fn bytes_to_text_preserves_lines_and_styles() {
        let text = bytes_to_text(b"\x1b[31mred\x1b[0m\nplain");
        let colored_span = text.lines[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "red")
            .unwrap();

        assert_eq!(text.lines.len(), 2);
        assert_eq!(colored_span.style.fg, Some(Color::Indexed(1)));
        assert_eq!(text.lines[1].spans[0].content.as_ref(), "plain");
        assert_eq!(text.lines[1].spans[0].style.fg, None);
    }

    #[test]
    fn bytes_to_text_tracks_multiple_sgr_modifiers() {
        let text = bytes_to_text(b"\x1b[1;3;4mstyled\x1b[0m");
        let style = text.lines[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "styled")
            .unwrap()
            .style;

        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn gen_ansi_all_set_str_applies_ansi_per_character() {
        let rows = gen_ansi_all_set_str("\x1b[32mab");
        let visible_spans: Vec<_> = rows[0]
            .iter()
            .filter(|span| !span.content.is_empty())
            .collect();

        assert_eq!(rows.len(), 1);
        assert_eq!(visible_spans.len(), 2);
        assert_eq!(visible_spans[0].content.as_ref(), "a");
        assert_eq!(visible_spans[1].content.as_ref(), "b");
        assert_eq!(visible_spans[0].style.fg, Some(Color::Indexed(2)));
        assert_eq!(visible_spans[1].style.fg, Some(Color::Indexed(2)));
    }

    #[test]
    fn get_ansi_strip_str_removes_escape_sequences() {
        let stripped = get_ansi_strip_str("\x1b[31mhello\x1b[0m world");

        assert_eq!(stripped, "hello world");
    }
}
