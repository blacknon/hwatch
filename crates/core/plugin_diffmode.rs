// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::slice;

use crate::common::parse_ansi_color;
use hwatch_ansi as ansi;
use hwatch_diffmode::{
    DiffMode, DiffModeOptions, DifferenceType, PluginDiffRequest, PluginMetadata, PluginOwnedBytes,
    PluginSlice, COLOR_BATCH_LINE_NUMBER_ADD, COLOR_BATCH_LINE_NUMBER_DEFAULT,
    COLOR_BATCH_LINE_NUMBER_REM, COLOR_WATCH_LINE_NUMBER_ADD, COLOR_WATCH_LINE_NUMBER_DEFAULT,
    COLOR_WATCH_LINE_NUMBER_REM, PLUGIN_ABI_VERSION, PLUGIN_OUTPUT_BATCH, PLUGIN_OUTPUT_WATCH,
};
use libloading::{Library, Symbol};
use serde::Deserialize;
use tui::{
    prelude::Line,
    style::{Color, Modifier, Style},
    text::Span,
};

const PLUGIN_RESPONSE_SCHEMA_V1: u32 = 1;
const PLUGIN_RESPONSE_SCHEMA_V2: u32 = 2;
const PLUGIN_RESPONSE_SCHEMA_V3: u32 = 3;

type PluginMetadataFn = unsafe extern "C" fn() -> PluginMetadata;
type PluginGenerateFn = unsafe extern "C" fn(PluginDiffRequest) -> PluginOwnedBytes;
type PluginFreeBytesFn = unsafe extern "C" fn(PluginOwnedBytes);

pub struct PluginRegistration {
    pub name: String,
    pub mode: Box<dyn DiffMode>,
}

pub fn load_plugin(path: &Path) -> Result<PluginRegistration, String> {
    PluginDiffMode::load(path).map(|mode| PluginRegistration {
        name: mode.plugin_name.clone(),
        mode: Box::new(mode),
    })
}

struct PluginDiffMode {
    _library: Library,
    generate: PluginGenerateFn,
    free_bytes: PluginFreeBytesFn,
    plugin_name: String,
    header_text: String,
    supports_only_diffline: bool,
    options: DiffModeOptions,
    plugin_path: PathBuf,
}

struct ParsedPluginDiffResponse {
    header_text: String,
    lines: Vec<PluginLine>,
    line_number_width: usize,
    use_core_gutter: bool,
}

#[derive(Deserialize)]
struct RawPluginDiffResponse {
    schema_version: u32,
    header_text: String,
    lines: Vec<RawPluginLine>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawPluginLine {
    Plain(String),
    Styled(PluginStyledLine),
}

enum PluginLine {
    Plain(String),
    Styled(PluginStyledLine),
}

#[derive(Deserialize)]
struct PluginStyledLine {
    #[serde(default)]
    line_no: Option<usize>,
    #[serde(default)]
    diff_type: Option<PluginDiffType>,
    #[serde(default)]
    gutter: Option<PluginGutterSpec>,
    spans: Vec<PluginStyledSpan>,
}

#[derive(Deserialize)]
struct PluginStyledSpan {
    text: String,
    #[serde(default)]
    style: PluginStyleSpec,
}

#[derive(Clone, Default, Deserialize)]
struct PluginStyleSpec {
    fg: Option<String>,
    bg: Option<String>,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    dim: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underlined: bool,
    #[serde(default)]
    reversed: bool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PluginDiffType {
    Same,
    Add,
    Rem,
}

impl PluginDiffType {
    fn as_difference_type(self) -> DifferenceType {
        match self {
            Self::Same => DifferenceType::Same,
            Self::Add => DifferenceType::Add,
            Self::Rem => DifferenceType::Rem,
        }
    }
}

#[derive(Default, Deserialize)]
struct PluginGutterSpec {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    style: PluginStyleSpec,
}

impl PluginDiffMode {
    fn load(path: &Path) -> Result<Self, String> {
        let library = unsafe { Library::new(path) }
            .map_err(|err| format!("failed to load plugin '{}': {err}", path.display()))?;

        let (generate, free_bytes, plugin_name, header_text, supports_only_diffline) = unsafe {
            let metadata_fn: Symbol<PluginMetadataFn> = library
                .get(b"hwatch_diffmode_metadata\0")
                .map_err(|err| format!("missing metadata symbol in '{}': {err}", path.display()))?;
            let generate_fn: Symbol<PluginGenerateFn> = library
                .get(b"hwatch_diffmode_generate\0")
                .map_err(|err| format!("missing generate symbol in '{}': {err}", path.display()))?;
            let free_bytes_fn: Symbol<PluginFreeBytesFn> = library
                .get(b"hwatch_diffmode_free_bytes\0")
                .map_err(|err| {
                    format!("missing free-bytes symbol in '{}': {err}", path.display())
                })?;

            let metadata = metadata_fn();
            validate_metadata(path, metadata)?;

            let plugin_name = cstr_to_string(metadata.plugin_name, "plugin_name", path)?;
            let header_text = cstr_to_string(metadata.header_text, "header_text", path)?;

            (
                *generate_fn,
                *free_bytes_fn,
                plugin_name,
                header_text,
                metadata.supports_only_diffline,
            )
        };

        Ok(Self {
            _library: library,
            generate,
            free_bytes,
            plugin_name,
            header_text,
            supports_only_diffline,
            options: DiffModeOptions::new(),
            plugin_path: path.to_path_buf(),
        })
    }

    fn invoke(
        &mut self,
        output_kind: u32,
        dest: &str,
        src: &str,
    ) -> Result<ParsedPluginDiffResponse, String> {
        let request = PluginDiffRequest {
            dest: PluginSlice {
                ptr: dest.as_ptr(),
                len: dest.len(),
            },
            src: PluginSlice {
                ptr: src.as_ptr(),
                len: src.len(),
            },
            output_kind,
            color: self.options.get_color(),
            line_number: self.options.get_line_number(),
            only_diffline: self.options.get_only_diffline(),
        };

        let bytes = unsafe { (self.generate)(request) };
        let response = parse_plugin_response(bytes, self.free_bytes, &self.plugin_path)?;
        self.header_text = response.header_text.clone();
        Ok(response)
    }

    fn error_lines(&self, message: String) -> Vec<String> {
        vec![format!(
            "plugin '{}' error: {message}",
            self.plugin_path.display()
        )]
    }
}

impl DiffMode for PluginDiffMode {
    fn generate_watch_diff(&mut self, dest: &str, src: &str) -> Vec<Line<'static>> {
        match self.invoke(PLUGIN_OUTPUT_WATCH, dest, src) {
            Ok(response) => {
                let use_core_gutter = response.use_core_gutter && self.options.get_line_number();
                let line_number_width = response.line_number_width;
                response
                    .lines
                    .into_iter()
                    .map(|line| {
                        render_watch_line(
                            line,
                            self.options.get_color(),
                            use_core_gutter,
                            line_number_width,
                        )
                    })
                    .collect()
            }
            Err(err) => self.error_lines(err).into_iter().map(Line::from).collect(),
        }
    }

    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String> {
        match self.invoke(PLUGIN_OUTPUT_BATCH, dest, src) {
            Ok(response) => {
                let use_core_gutter = response.use_core_gutter && self.options.get_line_number();
                let line_number_width = response.line_number_width;
                response
                    .lines
                    .into_iter()
                    .map(|line| {
                        render_batch_line(
                            line,
                            self.options.get_color(),
                            use_core_gutter,
                            line_number_width,
                        )
                    })
                    .collect()
            }
            Err(err) => self.error_lines(err),
        }
    }

    fn get_header_text(&self) -> String {
        if self.supports_only_diffline {
            if self.options.get_only_diffline() {
                if self.header_text.ends_with("(Only)") {
                    self.header_text.clone()
                } else {
                    format!("{}(Only)", self.header_text)
                }
            } else {
                self.header_text
                    .strip_suffix("(Only)")
                    .unwrap_or(&self.header_text)
                    .to_string()
            }
        } else {
            self.header_text.clone()
        }
    }

    fn get_support_only_diffline(&self) -> bool {
        self.supports_only_diffline
    }

    fn set_option(&mut self, options: DiffModeOptions) {
        self.options = options;
    }
}

unsafe fn cstr_to_string(
    ptr: *const std::ffi::c_char,
    field: &str,
    path: &Path,
) -> Result<String, String> {
    if ptr.is_null() {
        return Err(format!(
            "plugin '{}' returned null metadata field '{field}'",
            path.display()
        ));
    }

    CStr::from_ptr(ptr)
        .to_str()
        .map(|value| value.to_string())
        .map_err(|err| {
            format!(
                "plugin '{}' metadata field '{field}' is not valid UTF-8: {err}",
                path.display()
            )
        })
}

fn validate_metadata(path: &Path, metadata: PluginMetadata) -> Result<(), String> {
    if metadata.abi_version != PLUGIN_ABI_VERSION {
        return Err(format!(
            "plugin '{}' ABI mismatch: expected {}, got {}",
            path.display(),
            PLUGIN_ABI_VERSION,
            metadata.abi_version
        ));
    }

    Ok(())
}

fn parse_plugin_response(
    bytes: PluginOwnedBytes,
    free_bytes: PluginFreeBytesFn,
    path: &Path,
) -> Result<ParsedPluginDiffResponse, String> {
    if bytes.ptr.is_null() {
        return Err(format!(
            "plugin '{}' returned null response bytes",
            path.display()
        ));
    }

    let json_bytes = unsafe {
        let raw = slice::from_raw_parts(bytes.ptr, bytes.len).to_vec();
        free_bytes(bytes);
        raw
    };
    let json = String::from_utf8(json_bytes)
        .map_err(|err| format!("plugin '{}' returned invalid UTF-8: {err}", path.display()))?;

    let response: RawPluginDiffResponse = serde_json::from_str(&json)
        .map_err(|err| format!("plugin '{}' returned invalid JSON: {err}", path.display()))?;

    if response.schema_version != PLUGIN_RESPONSE_SCHEMA_V1
        && response.schema_version != PLUGIN_RESPONSE_SCHEMA_V2
        && response.schema_version != PLUGIN_RESPONSE_SCHEMA_V3
    {
        return Err(format!(
            "plugin '{}' response schema mismatch: expected {}, {}, or {}, got {}",
            path.display(),
            PLUGIN_RESPONSE_SCHEMA_V1,
            PLUGIN_RESPONSE_SCHEMA_V2,
            PLUGIN_RESPONSE_SCHEMA_V3,
            response.schema_version
        ));
    }

    let use_core_gutter = response.schema_version >= PLUGIN_RESPONSE_SCHEMA_V3;
    let mut lines = Vec::with_capacity(response.lines.len());
    for (line_index, line) in response.lines.into_iter().enumerate() {
        match line {
            RawPluginLine::Plain(text) => lines.push(PluginLine::Plain(text)),
            RawPluginLine::Styled(styled) => {
                validate_styled_line(path, line_index, &styled)?;
                lines.push(PluginLine::Styled(styled));
            }
        }
    }

    let line_number_width = if use_core_gutter {
        lines
            .iter()
            .filter_map(|line| match line {
                PluginLine::Plain(_) => None,
                PluginLine::Styled(line) => line.line_no,
            })
            .max()
            .unwrap_or(0)
            .to_string()
            .len()
            .max(1)
    } else {
        0
    };

    Ok(ParsedPluginDiffResponse {
        header_text: response.header_text,
        lines,
        line_number_width,
        use_core_gutter,
    })
}

fn validate_styled_line(
    path: &Path,
    line_index: usize,
    line: &PluginStyledLine,
) -> Result<(), String> {
    if let Some(gutter) = &line.gutter {
        validate_style_color(path, line_index, 0, "gutter.fg", gutter.style.fg.as_deref())?;
        validate_style_color(path, line_index, 0, "gutter.bg", gutter.style.bg.as_deref())?;
    }
    for (span_index, span) in line.spans.iter().enumerate() {
        validate_style_color(path, line_index, span_index, "fg", span.style.fg.as_deref())?;
        validate_style_color(path, line_index, span_index, "bg", span.style.bg.as_deref())?;
    }
    Ok(())
}

fn validate_style_color(
    path: &Path,
    line_index: usize,
    span_index: usize,
    field: &str,
    value: Option<&str>,
) -> Result<(), String> {
    if let Some(value) = value {
        parse_ansi_color(value).map_err(|err| {
            format!(
                "plugin '{}' line {} span {} has invalid style {} '{}': {}",
                path.display(),
                line_index + 1,
                span_index + 1,
                field,
                value,
                err
            )
        })?;
    }
    Ok(())
}

fn render_watch_line(
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

fn render_batch_line(
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
