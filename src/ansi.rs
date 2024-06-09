// Copyright (c) 2024 Blacknon.
// This code from https://github.com/blacknon/ansi4tui/blob/master/src/lib.rs

use ansi_parser::{AnsiParser, AnsiSequence, Output};
use termwiz::cell::{Blink, Intensity, Underline};
use termwiz::color::ColorSpec;
use termwiz::escape::{
    csi::{Sgr, CSI},
    parser::Parser,
    Action, ControlCode,
};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span,Text};
use tui::prelude::Line;

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
            Action::Control(ControlCode::LineFeed) => {
                // finish the current span
                current_line.push(Span::styled(span_text, span_style));
                span_text = String::new();

                // finish the current line
                spans.push(Line::from(current_line));
                current_line = Vec::new();
            }
            Action::CSI(CSI::Sgr(sgr)) => {
                // ignore this single condition, for the rest we'll close the current span
                if let Sgr::Font(_) = sgr {
                    continue;
                }

                // finish the current span
                current_line.push(Span::styled(span_text, span_style));
                span_text = String::new();

                match sgr {
                    Sgr::Reset => span_style = Style::default(),
                    Sgr::Intensity(i) => match i {
                        Intensity::Bold => {
                            span_style = span_style.remove_modifier(Modifier::DIM);
                            span_style = span_style.add_modifier(Modifier::BOLD);
                        }
                        Intensity::Half => {
                            span_style = span_style.add_modifier(Modifier::DIM);
                            span_style = span_style.remove_modifier(Modifier::BOLD);
                        }
                        Intensity::Normal => {
                            span_style = span_style.remove_modifier(Modifier::DIM);
                            span_style = span_style.remove_modifier(Modifier::BOLD);
                        }
                    },
                    Sgr::Underline(u) => match u {
                        Underline::Double | Underline::Single => {
                            span_style = span_style.add_modifier(Modifier::UNDERLINED);
                        }
                        _ => span_style = span_style.remove_modifier(Modifier::UNDERLINED),
                    },
                    Sgr::Blink(b) => match b {
                        Blink::Slow => {
                            span_style = span_style.add_modifier(Modifier::SLOW_BLINK);
                            span_style = span_style.remove_modifier(Modifier::RAPID_BLINK);
                        }
                        Blink::Rapid => {
                            span_style = span_style.remove_modifier(Modifier::SLOW_BLINK);
                            span_style = span_style.add_modifier(Modifier::RAPID_BLINK);
                        }
                        Blink::None => {
                            span_style = span_style.remove_modifier(Modifier::SLOW_BLINK);
                            span_style = span_style.remove_modifier(Modifier::RAPID_BLINK);
                        }
                    },
                    Sgr::Italic(true) => span_style = span_style.add_modifier(Modifier::ITALIC),
                    Sgr::Italic(false) => span_style = span_style.remove_modifier(Modifier::ITALIC),
                    Sgr::Inverse(true) => span_style = span_style.add_modifier(Modifier::REVERSED),
                    Sgr::Inverse(false) => {
                        span_style = span_style.remove_modifier(Modifier::REVERSED)
                    }
                    Sgr::Invisible(true) => span_style = span_style.add_modifier(Modifier::HIDDEN),
                    Sgr::Invisible(false) => {
                        span_style = span_style.remove_modifier(Modifier::HIDDEN)
                    }
                    Sgr::StrikeThrough(true) => {
                        span_style = span_style.add_modifier(Modifier::CROSSED_OUT)
                    }
                    Sgr::StrikeThrough(false) => {
                        span_style = span_style.remove_modifier(Modifier::CROSSED_OUT)
                    }
                    Sgr::Foreground(c) => match c {
                        ColorSpec::Default => span_style = span_style.fg(Color::Reset),
                        ColorSpec::PaletteIndex(i) => span_style = span_style.fg(Color::Indexed(i)),
                        ColorSpec::TrueColor(rgb) => {
                            let rgb_tuple = rgb.to_srgb_u8();
                            span_style =
                                span_style.bg(Color::Rgb(rgb_tuple.0, rgb_tuple.1, rgb_tuple.2));
                        }
                    },
                    Sgr::Background(c) => match c {
                        ColorSpec::Default => span_style = span_style.bg(Color::Reset),
                        ColorSpec::PaletteIndex(i) => span_style = span_style.bg(Color::Indexed(i)),
                        ColorSpec::TrueColor(rgb) => {
                            let rgb_tuple = rgb.to_srgb_u8();
                            span_style =
                                span_style.bg(Color::Rgb(rgb_tuple.0, rgb_tuple.1, rgb_tuple.2));
                        },
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // push any remaining data
    if !span_text.is_empty() {
        // finish the current span
        current_line.push(Span::styled(span_text, span_style));
        // finish the current line
        spans.push(Line::from(current_line));
    }

    spans.into()
}

// Ansi Color Code parse
// ==========

/// Apply ANSI color code character by character.
pub fn gen_ansi_all_set_str<'b>(text: &str) -> Vec<Vec<Span<'b>>> {
    // set Result
    let mut result = vec![];

    // ansi reset code heapless_vec
    let mut ansi_reset_vec = heapless::Vec::<u8, 5>::new();
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
                    let data = bytes_to_text(format!("{append_text}\n").as_bytes());
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
pub fn get_ansi_strip_str(text: &str) -> String {
    let mut line_str = "".to_string();
    for block in text.ansi_parse() {
        if let Output::TextBlock(t) = block {
            line_str.push_str(t);
        }
    }

    line_str
}
