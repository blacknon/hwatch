// Copyright (c) 2026 Blacknon.
// This code from https://github.com/blacknon/ansi4tui/blob/master/src/lib.rs

use tui::prelude::Line;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Text};

use ratatui as tui;

fn apply_extended_color(params: &[Option<u16>]) -> Option<(Color, usize)> {
    match params.first().copied().flatten() {
        Some(5) => params
            .get(1)
            .and_then(|value| value.map(|index| (Color::Indexed(index as u8), 2))),
        Some(2) => {
            let r = params.get(1).copied().flatten()?;
            let g = params.get(2).copied().flatten()?;
            let b = params.get(3).copied().flatten()?;
            Some((Color::Rgb(r as u8, g as u8, b as u8), 4))
        }
        _ => None,
    }
}

fn apply_sgr_params(span_style: &mut Style, params: &[Option<u16>]) {
    let params = if params.is_empty() {
        vec![Some(0)]
    } else {
        params.to_vec()
    };

    let mut index = 0;
    while index < params.len() {
        let code = params[index].unwrap_or(0);
        match code {
            0 => *span_style = Style::default(),
            1 => {
                *span_style = span_style.remove_modifier(Modifier::DIM);
                *span_style = span_style.add_modifier(Modifier::BOLD);
            }
            2 => {
                *span_style = span_style.add_modifier(Modifier::DIM);
                *span_style = span_style.remove_modifier(Modifier::BOLD);
            }
            3 => *span_style = span_style.add_modifier(Modifier::ITALIC),
            4 | 21 => *span_style = span_style.add_modifier(Modifier::UNDERLINED),
            5 => {
                *span_style = span_style.add_modifier(Modifier::SLOW_BLINK);
                *span_style = span_style.remove_modifier(Modifier::RAPID_BLINK);
            }
            6 => {
                *span_style = span_style.remove_modifier(Modifier::SLOW_BLINK);
                *span_style = span_style.add_modifier(Modifier::RAPID_BLINK);
            }
            7 => *span_style = span_style.add_modifier(Modifier::REVERSED),
            8 => *span_style = span_style.add_modifier(Modifier::HIDDEN),
            9 => *span_style = span_style.add_modifier(Modifier::CROSSED_OUT),
            22 => {
                *span_style = span_style.remove_modifier(Modifier::DIM);
                *span_style = span_style.remove_modifier(Modifier::BOLD);
            }
            23 => *span_style = span_style.remove_modifier(Modifier::ITALIC),
            24 => *span_style = span_style.remove_modifier(Modifier::UNDERLINED),
            25 => {
                *span_style = span_style.remove_modifier(Modifier::SLOW_BLINK);
                *span_style = span_style.remove_modifier(Modifier::RAPID_BLINK);
            }
            27 => *span_style = span_style.remove_modifier(Modifier::REVERSED),
            28 => *span_style = span_style.remove_modifier(Modifier::HIDDEN),
            29 => *span_style = span_style.remove_modifier(Modifier::CROSSED_OUT),
            30..=37 => *span_style = span_style.fg(Color::Indexed((code - 30) as u8)),
            38 => {
                if let Some((color, consumed)) = apply_extended_color(&params[index + 1..]) {
                    *span_style = span_style.fg(color);
                    index += consumed;
                }
            }
            39 => *span_style = span_style.fg(Color::Reset),
            40..=47 => *span_style = span_style.bg(Color::Indexed((code - 40) as u8)),
            48 => {
                if let Some((color, consumed)) = apply_extended_color(&params[index + 1..]) {
                    *span_style = span_style.bg(color);
                    index += consumed;
                }
            }
            49 => *span_style = span_style.bg(Color::Reset),
            90..=97 => *span_style = span_style.fg(Color::Indexed((code - 82) as u8)),
            100..=107 => *span_style = span_style.bg(Color::Indexed((code - 92) as u8)),
            _ => {}
        }

        index += 1;
    }
}

fn parse_csi_sequence(chars: &[char], index: &mut usize) -> Option<(char, Vec<Option<u16>>)> {
    if chars.get(*index) != Some(&'\x1b') || chars.get(*index + 1) != Some(&'[') {
        return None;
    }

    *index += 2;
    let mut params = String::new();
    while let Some(ch) = chars.get(*index).copied() {
        *index += 1;
        if ('@'..='~').contains(&ch) {
            let parsed = if params.is_empty() {
                Vec::new()
            } else {
                params
                    .split(';')
                    .map(|part| {
                        if part.is_empty() {
                            None
                        } else {
                            part.parse::<u16>().ok()
                        }
                    })
                    .collect()
            };
            return Some((ch, parsed));
        }

        params.push(ch);
    }

    None
}

fn push_current_span<'a>(spans: &mut Vec<Span<'a>>, text: &mut String, style: Style) {
    if !text.is_empty() {
        spans.push(Span::styled(std::mem::take(text), style));
    }
}

/// Converts ANSI-escaped strings to tui-rs compatible text
pub fn bytes_to_text<'a, B: AsRef<[u8]>>(bytes: B) -> Text<'a> {
    let text = String::from_utf8_lossy(bytes.as_ref());
    let chars: Vec<char> = text.chars().collect();

    let mut lines = Vec::<Line>::new();
    let mut span_style = Style::default();
    let mut span_text = String::new();
    let mut current_line = Vec::<Span>::new();

    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            '\x1b' => {
                push_current_span(&mut current_line, &mut span_text, span_style);
                if let Some((action, params)) = parse_csi_sequence(&chars, &mut index) {
                    if action == 'm' {
                        apply_sgr_params(&mut span_style, &params);
                    }
                } else {
                    index += 1;
                }
            }
            '\n' => {
                push_current_span(&mut current_line, &mut span_text, span_style);
                lines.push(Line::from(current_line));
                current_line = Vec::new();
                index += 1;
            }
            '\r' => {
                index += 1;
            }
            ch => {
                span_text.push(ch);
                index += 1;
            }
        }
    }

    push_current_span(&mut current_line, &mut span_text, span_style);
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    lines.into()
}

// Ansi Color Code parse
// ==========

/// Apply ANSI color code character by character.
pub fn gen_ansi_all_set_str<'b>(text: &str) -> Vec<Vec<Span<'b>>> {
    let chars: Vec<char> = text.chars().collect();
    let mut result = vec![Vec::new()];
    let mut span_style = Style::default();

    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            '\x1b' => {
                if let Some((action, params)) = parse_csi_sequence(&chars, &mut index) {
                    if action == 'm' {
                        apply_sgr_params(&mut span_style, &params);
                    }
                } else {
                    index += 1;
                }
            }
            '\n' => {
                result.push(Vec::new());
                index += 1;
            }
            '\r' => {
                index += 1;
            }
            ch => {
                result
                    .last_mut()
                    .unwrap()
                    .push(Span::styled(ch.to_string(), span_style));
                index += 1;
            }
        }
    }

    result
}

pub fn get_ansi_strip_str(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut line_str = String::new();

    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            '\x1b' => {
                if parse_csi_sequence(&chars, &mut index).is_none() {
                    index += 1;
                }
            }
            '\r' => {
                index += 1;
            }
            ch => {
                line_str.push(ch);
                index += 1;
            }
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
    fn bytes_to_text_supports_rgb_styles() {
        let text = bytes_to_text(b"\x1b[38;2;1;2;3mcolor\x1b[0m");
        let colored_span = text.lines[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "color")
            .unwrap();

        assert_eq!(colored_span.style.fg, Some(Color::Rgb(1, 2, 3)));
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

    #[test]
    fn get_ansi_strip_str_ignores_non_sgr_csi_sequences() {
        let stripped = get_ansi_strip_str("a\x1b[2Kb\x1b[1;1Hc");

        assert_eq!(stripped, "abc");
    }
}
