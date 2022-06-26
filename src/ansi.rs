// Copyright (c) 2021 Blacknon.
// This code from https://github.com/blacknon/ansi4tui/blob/master/src/lib.rs

use termwiz::cell::{Blink, Intensity, Underline};
use termwiz::color::ColorSpec;
use termwiz::escape::{
    csi::{Sgr, CSI},
    parser::Parser,
    Action, ControlCode,
};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};

/// Converts ANSI-escaped strings to tui-rs compatible text
pub fn bytes_to_text<'a, B: AsRef<[u8]>>(bytes: B) -> Text<'a> {
    let mut parser = Parser::new();
    let parsed = parser.parse_as_vec(bytes.as_ref());

    // each span will be a line
    let mut spans = Vec::<Spans>::new();

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
                spans.push(Spans::from(current_line));
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
                            let rgb_tuple = rgb.to_tuple_rgb8();
                            span_style =
                                span_style.fg(Color::Rgb(rgb_tuple.0, rgb_tuple.1, rgb_tuple.2));
                        }
                    },
                    Sgr::Background(c) => match c {
                        ColorSpec::Default => span_style = span_style.bg(Color::Reset),
                        ColorSpec::PaletteIndex(i) => span_style = span_style.bg(Color::Indexed(i)),
                        ColorSpec::TrueColor(rgb) => {
                            let rgb_tuple = rgb.to_tuple_rgb8();
                            span_style =
                                span_style.bg(Color::Rgb(rgb_tuple.0, rgb_tuple.1, rgb_tuple.2));
                        }
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
        spans.push(Spans::from(current_line));
    }

    spans.into()
}
