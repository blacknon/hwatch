// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::types::{PluginDiffType, PluginGutterSpec, PluginLine, PluginStyleSpec};
use crate::common::parse_ansi_color;
use hwatch_ansi as ansi;
use hwatch_diffmode::{
    DifferenceType, COLOR_BATCH_LINE_NUMBER_ADD, COLOR_BATCH_LINE_NUMBER_DEFAULT,
    COLOR_BATCH_LINE_NUMBER_REM, COLOR_WATCH_LINE_NUMBER_ADD, COLOR_WATCH_LINE_NUMBER_DEFAULT,
    COLOR_WATCH_LINE_NUMBER_REM,
};
use tui::{
    prelude::Line,
    style::{Color, Modifier, Style},
    text::Span,
};

pub(super) fn render_watch_line(
    line: PluginLine,
    use_color: bool,
    use_core_gutter: bool,
    line_number_width: usize,
) -> Line<'static> {
    let gutter = match &line {
        PluginLine::Styled(line) if use_core_gutter => Some(rendered_gutter(
            line.line_no,
            line.diff_type
                .unwrap_or(PluginDiffType::Same)
                .as_difference_type(),
            line.gutter.as_ref(),
            line_number_width,
        )),
        _ => None,
    };

    let mut rendered = match line {
        PluginLine::Plain(text) => {
            if use_color {
                ansi::bytes_to_text(format!("{text}\n").as_bytes())
                    .lines
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Line::from(String::new()))
            } else {
                Line::from(ansi::get_ansi_strip_str(&text))
            }
        }
        PluginLine::Styled(line) => {
            let mut rendered_spans = Vec::new();
            for span in line.spans {
                let style = to_tui_style(&span.style);
                if use_color && span.text.contains('\u{1b}') {
                    let ansi_line = ansi::bytes_to_text(format!("{}\n", span.text).as_bytes())
                        .lines
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Line::from(String::new()));
                    for ansi_span in ansi_line.spans {
                        rendered_spans.push(Span::styled(
                            ansi_span.content.into_owned(),
                            ansi_span.style.patch(style),
                        ));
                    }
                } else if !use_color && span.text.contains('\u{1b}') {
                    rendered_spans.push(Span::styled(ansi::get_ansi_strip_str(&span.text), style));
                } else {
                    rendered_spans.push(Span::styled(span.text, style));
                }
            }
            Line::from(rendered_spans)
        }
    };

    if let Some(gutter) = gutter {
        rendered
            .spans
            .insert(0, Span::styled(gutter.text, gutter.watch_style));
    }

    rendered
}

pub(super) fn render_batch_line(
    line: PluginLine,
    use_color: bool,
    use_core_gutter: bool,
    line_number_width: usize,
) -> String {
    let gutter = match &line {
        PluginLine::Styled(line) if use_core_gutter => Some(rendered_gutter(
            line.line_no,
            line.diff_type
                .unwrap_or(PluginDiffType::Same)
                .as_difference_type(),
            line.gutter.as_ref(),
            line_number_width,
        )),
        _ => None,
    };

    let mut rendered = match line {
        PluginLine::Plain(text) => {
            if use_color {
                text
            } else {
                ansi::get_ansi_strip_str(&text)
            }
        }
        PluginLine::Styled(line) => {
            let mut rendered = String::new();
            for span in line.spans {
                let style = to_ansi_style(&span.style);
                if use_color && span.text.contains('\u{1b}') {
                    let ansi_line = ansi::bytes_to_text(format!("{}\n", span.text).as_bytes())
                        .lines
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Line::from(String::new()));
                    for ansi_span in ansi_line.spans {
                        rendered.push_str(
                            &to_ansi_style_from_tui(
                                &ansi_span.style.patch(to_tui_style(&span.style)),
                            )
                            .paint(ansi_span.content.into_owned())
                            .to_string(),
                        );
                    }
                } else {
                    let text = if use_color {
                        span.text
                    } else {
                        ansi::get_ansi_strip_str(&span.text)
                    };
                    rendered.push_str(&style.paint(text).to_string());
                }
            }
            rendered
        }
    };

    if let Some(gutter) = gutter {
        let prefix = if use_color {
            gutter.ansi_style.paint(gutter.text).to_string()
        } else {
            gutter.text
        };
        rendered.insert_str(0, &prefix);
    }

    rendered
}

struct RenderedGutter {
    text: String,
    watch_style: Style,
    ansi_style: ansi_term::Style,
}

fn rendered_gutter(
    line_no: Option<usize>,
    diff_type: DifferenceType,
    gutter: Option<&PluginGutterSpec>,
    line_number_width: usize,
) -> RenderedGutter {
    let mut watch_style = default_watch_gutter_style(&diff_type);
    let mut ansi_style = default_ansi_gutter_style(&diff_type);
    let default_text = match line_no {
        Some(line_no) => format!("{line_no:>line_number_width$} | "),
        None => format!("{:>line_number_width$} | ", ""),
    };

    if let Some(gutter) = gutter {
        watch_style = patch_tui_style(watch_style, &gutter.style);
        ansi_style = patch_ansi_style(ansi_style, &gutter.style);
        return RenderedGutter {
            text: gutter.text.clone().unwrap_or(default_text),
            watch_style,
            ansi_style,
        };
    }

    RenderedGutter {
        text: default_text,
        watch_style,
        ansi_style,
    }
}

fn to_tui_style(spec: &PluginStyleSpec) -> Style {
    patch_tui_style(Style::default(), spec)
}

fn to_ansi_style(spec: &PluginStyleSpec) -> ansi_term::Style {
    patch_ansi_style(ansi_term::Style::new(), spec)
}

fn patch_tui_style(mut style: Style, spec: &PluginStyleSpec) -> Style {
    if let Some(fg) = spec.fg.as_deref().and_then(parse_color_lossy) {
        style = style.fg(fg);
    }
    if let Some(bg) = spec.bg.as_deref().and_then(parse_color_lossy) {
        style = style.bg(bg);
    }
    if spec.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if spec.dim {
        style = style.add_modifier(Modifier::DIM);
    }
    if spec.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if spec.underlined {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if spec.reversed {
        style = style.add_modifier(Modifier::REVERSED);
    }
    style
}

fn patch_ansi_style(mut style: ansi_term::Style, spec: &PluginStyleSpec) -> ansi_term::Style {
    if let Some(fg) = spec.fg.as_deref().and_then(parse_ansi_colour_lossy) {
        style = style.fg(fg);
    }
    if let Some(bg) = spec.bg.as_deref().and_then(parse_ansi_colour_lossy) {
        style = style.on(bg);
    }
    if spec.bold {
        style = style.bold();
    }
    if spec.dim {
        style = style.dimmed();
    }
    if spec.italic {
        style = style.italic();
    }
    if spec.underlined {
        style = style.underline();
    }
    if spec.reversed {
        style = style.reverse();
    }
    style
}

fn default_watch_gutter_style(diff_type: &DifferenceType) -> Style {
    let color = match diff_type {
        DifferenceType::Same => COLOR_WATCH_LINE_NUMBER_DEFAULT,
        DifferenceType::Add => COLOR_WATCH_LINE_NUMBER_ADD,
        DifferenceType::Rem => COLOR_WATCH_LINE_NUMBER_REM,
    };
    Style::default().fg(color)
}

fn default_ansi_gutter_style(diff_type: &DifferenceType) -> ansi_term::Style {
    let color = match diff_type {
        DifferenceType::Same => COLOR_BATCH_LINE_NUMBER_DEFAULT,
        DifferenceType::Add => COLOR_BATCH_LINE_NUMBER_ADD,
        DifferenceType::Rem => COLOR_BATCH_LINE_NUMBER_REM,
    };
    ansi_term::Style::default().fg(color)
}

fn parse_color_lossy(value: &str) -> Option<Color> {
    parse_ansi_color(value).ok()
}

fn parse_ansi_colour_lossy(value: &str) -> Option<ansi_term::Colour> {
    match parse_ansi_color(value).ok()? {
        Color::Black => Some(ansi_term::Colour::Black),
        Color::Red => Some(ansi_term::Colour::Red),
        Color::Green => Some(ansi_term::Colour::Green),
        Color::Yellow => Some(ansi_term::Colour::Yellow),
        Color::Blue => Some(ansi_term::Colour::Blue),
        Color::Magenta => Some(ansi_term::Colour::Purple),
        Color::Cyan => Some(ansi_term::Colour::Cyan),
        Color::Gray => Some(ansi_term::Colour::White),
        Color::DarkGray => Some(ansi_term::Colour::Fixed(8)),
        Color::LightRed => Some(ansi_term::Colour::Fixed(9)),
        Color::LightGreen => Some(ansi_term::Colour::Fixed(10)),
        Color::LightYellow => Some(ansi_term::Colour::Fixed(11)),
        Color::LightBlue => Some(ansi_term::Colour::Fixed(12)),
        Color::LightMagenta => Some(ansi_term::Colour::Fixed(13)),
        Color::LightCyan => Some(ansi_term::Colour::Fixed(14)),
        Color::White => Some(ansi_term::Colour::White),
        Color::Rgb(r, g, b) => Some(ansi_term::Colour::RGB(r, g, b)),
        Color::Indexed(index) => Some(ansi_term::Colour::Fixed(index)),
        Color::Reset => None,
    }
}

fn to_ansi_style_from_tui(style: &Style) -> ansi_term::Style {
    let mut ansi_style = ansi_term::Style::new();

    if let Some(fg) = style.fg.and_then(color_to_ansi_colour_lossy) {
        ansi_style = ansi_style.fg(fg);
    }
    if let Some(bg) = style.bg.and_then(color_to_ansi_colour_lossy) {
        ansi_style = ansi_style.on(bg);
    }
    if style.add_modifier.contains(Modifier::BOLD) {
        ansi_style = ansi_style.bold();
    }
    if style.add_modifier.contains(Modifier::DIM) {
        ansi_style = ansi_style.dimmed();
    }
    if style.add_modifier.contains(Modifier::ITALIC) {
        ansi_style = ansi_style.italic();
    }
    if style.add_modifier.contains(Modifier::UNDERLINED) {
        ansi_style = ansi_style.underline();
    }
    if style.add_modifier.contains(Modifier::REVERSED) {
        ansi_style = ansi_style.reverse();
    }

    ansi_style
}

fn color_to_ansi_colour_lossy(color: Color) -> Option<ansi_term::Colour> {
    match color {
        Color::Black => Some(ansi_term::Colour::Black),
        Color::Red => Some(ansi_term::Colour::Red),
        Color::Green => Some(ansi_term::Colour::Green),
        Color::Yellow => Some(ansi_term::Colour::Yellow),
        Color::Blue => Some(ansi_term::Colour::Blue),
        Color::Magenta => Some(ansi_term::Colour::Purple),
        Color::Cyan => Some(ansi_term::Colour::Cyan),
        Color::Gray => Some(ansi_term::Colour::White),
        Color::DarkGray => Some(ansi_term::Colour::Fixed(8)),
        Color::LightRed => Some(ansi_term::Colour::Fixed(9)),
        Color::LightGreen => Some(ansi_term::Colour::Fixed(10)),
        Color::LightYellow => Some(ansi_term::Colour::Fixed(11)),
        Color::LightBlue => Some(ansi_term::Colour::Fixed(12)),
        Color::LightMagenta => Some(ansi_term::Colour::Fixed(13)),
        Color::LightCyan => Some(ansi_term::Colour::Fixed(14)),
        Color::White => Some(ansi_term::Colour::White),
        Color::Rgb(r, g, b) => Some(ansi_term::Colour::RGB(r, g, b)),
        Color::Indexed(index) => Some(ansi_term::Colour::Fixed(index)),
        Color::Reset => None,
    }
}
