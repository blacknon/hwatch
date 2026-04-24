// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::slice;

use crate::common::parse_ansi_color;
use hwatch_ansi as ansi;
use hwatch_diffmode::{
    DiffMode, DiffModeOptions, DifferenceType, PluginDiffRequest, PluginDiffRequestV1,
    PluginMetadata, PluginOwnedBytes, PluginSlice, COLOR_BATCH_LINE_NUMBER_ADD,
    COLOR_BATCH_LINE_NUMBER_DEFAULT, COLOR_BATCH_LINE_NUMBER_REM, COLOR_WATCH_LINE_NUMBER_ADD,
    COLOR_WATCH_LINE_NUMBER_DEFAULT, COLOR_WATCH_LINE_NUMBER_REM, PLUGIN_ABI_VERSION,
    PLUGIN_ABI_VERSION_V1, PLUGIN_OUTPUT_BATCH, PLUGIN_OUTPUT_WATCH,
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
const MAX_PLUGIN_RESPONSE_BYTES: usize = 64 * 1024 * 1024;

type PluginMetadataFn = unsafe extern "C" fn() -> PluginMetadata;
type PluginGenerateFnV1 = unsafe extern "C" fn(PluginDiffRequestV1) -> PluginOwnedBytes;
type PluginGenerateFnV2 = unsafe extern "C" fn(PluginDiffRequest) -> PluginOwnedBytes;
type PluginFreeBytesFn = unsafe extern "C" fn(PluginOwnedBytes);

enum PluginGenerateFn {
    V1(PluginGenerateFnV1),
    V2(PluginGenerateFnV2),
}

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
    _library: Option<Library>,
    generate: PluginGenerateFn,
    free_bytes: PluginFreeBytesFn,
    plugin_name: String,
    header_text: String,
    supports_only_diffline: bool,
    options: DiffModeOptions,
    plugin_path: PathBuf,
}

#[derive(Debug)]
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

#[derive(Debug)]
enum PluginLine {
    Plain(String),
    Styled(PluginStyledLine),
}

#[derive(Debug, Deserialize)]
struct PluginStyledLine {
    #[serde(default)]
    line_no: Option<usize>,
    #[serde(default)]
    diff_type: Option<PluginDiffType>,
    #[serde(default)]
    gutter: Option<PluginGutterSpec>,
    spans: Vec<PluginStyledSpan>,
}

#[derive(Debug, Deserialize)]
struct PluginStyledSpan {
    text: String,
    #[serde(default)]
    style: PluginStyleSpec,
}

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Copy, Debug, Deserialize)]
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

#[derive(Debug, Default, Deserialize)]
struct PluginGutterSpec {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    style: PluginStyleSpec,
}

#[derive(Debug)]
struct ValidatedPluginMetadata {
    abi_version: u32,
    supports_only_diffline: bool,
    plugin_name: String,
    header_text: String,
}

impl PluginDiffMode {
    fn load(path: &Path) -> Result<Self, String> {
        let library = unsafe { Library::new(path) }
            .map_err(|err| plugin_load_error(path, format!("{err}")))?;

        let (generate, free_bytes, metadata) =
            unsafe {
                let metadata_fn: Symbol<PluginMetadataFn> =
                    library.get(b"hwatch_diffmode_metadata\0").map_err(|err| {
                        plugin_metadata_error(
                            path,
                            format!("missing exported symbol 'hwatch_diffmode_metadata': {err}"),
                        )
                    })?;
                let free_bytes_fn: Symbol<PluginFreeBytesFn> = library
                    .get(b"hwatch_diffmode_free_bytes\0")
                    .map_err(|err| {
                        plugin_metadata_error(
                            path,
                            format!("missing exported symbol 'hwatch_diffmode_free_bytes': {err}"),
                        )
                    })?;

                let metadata = metadata_fn();
                let metadata = validate_metadata(path, metadata)?;

                let generate_fn =
                    match metadata.abi_version {
                        PLUGIN_ABI_VERSION_V1 => PluginGenerateFn::V1(
                            *library
                                .get::<PluginGenerateFnV1>(b"hwatch_diffmode_generate\0")
                                .map_err(|err| {
                                    plugin_metadata_error(
                        path,
                        format!("missing exported symbol 'hwatch_diffmode_generate': {err}"),
                    )
                                })?,
                        ),
                        PLUGIN_ABI_VERSION => PluginGenerateFn::V2(
                            *library
                                .get::<PluginGenerateFnV2>(b"hwatch_diffmode_generate\0")
                                .map_err(|err| {
                                    plugin_metadata_error(
                        path,
                        format!("missing exported symbol 'hwatch_diffmode_generate': {err}"),
                    )
                                })?,
                        ),
                        _ => unreachable!(),
                    };

                (generate_fn, *free_bytes_fn, metadata)
            };

        Ok(Self {
            _library: Some(library),
            generate,
            free_bytes,
            plugin_name: metadata.plugin_name,
            header_text: metadata.header_text,
            supports_only_diffline: metadata.supports_only_diffline,
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
        let dest = PluginSlice {
            ptr: dest.as_ptr(),
            len: dest.len(),
        };
        let src = PluginSlice {
            ptr: src.as_ptr(),
            len: src.len(),
        };

        let bytes = unsafe {
            match self.generate {
                PluginGenerateFn::V1(generate) => generate(PluginDiffRequestV1 {
                    dest,
                    src,
                    output_kind,
                    color: self.options.get_color(),
                    line_number: self.options.get_line_number(),
                    only_diffline: self.options.get_only_diffline(),
                }),
                PluginGenerateFn::V2(generate) => generate(PluginDiffRequest {
                    dest,
                    src,
                    output_kind,
                    color: self.options.get_color(),
                    line_number: self.options.get_line_number(),
                    only_diffline: self.options.get_only_diffline(),
                    ignore_spaceblock: self.options.get_ignore_spaceblock(),
                }),
            }
        };
        let response = parse_plugin_response(bytes, self.free_bytes, &self.plugin_path)?;
        self.header_text = response.header_text.clone();
        Ok(response)
    }

    fn error_lines(&self, message: String) -> Vec<String> {
        vec![plugin_runtime_error(&self.plugin_path, message)]
    }
}

fn plugin_load_error(path: &Path, detail: impl Into<String>) -> String {
    format!("plugin '{}' load failed: {}", path.display(), detail.into())
}

fn plugin_error_at_stage(path: &Path, stage: &str, detail: impl Into<String>) -> String {
    format!("plugin '{}' {}: {}", path.display(), stage, detail.into())
}

fn plugin_metadata_error(path: &Path, detail: impl Into<String>) -> String {
    plugin_error_at_stage(path, "metadata", detail)
}

fn plugin_response_error(path: &Path, detail: impl Into<String>) -> String {
    plugin_error_at_stage(path, "response", detail)
}

fn plugin_response_bytes_error(path: &Path, detail: impl Into<String>) -> String {
    format!(
        "plugin '{}' response bytes: {}",
        path.display(),
        detail.into()
    )
}

fn plugin_runtime_error(path: &Path, detail: impl Into<String>) -> String {
    format!("plugin '{}' error: {}", path.display(), detail.into())
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
        return Err(plugin_metadata_error(
            path,
            format!("returned null field '{field}'"),
        ));
    }

    CStr::from_ptr(ptr)
        .to_str()
        .map(|value| value.to_string())
        .map_err(|err| {
            plugin_metadata_error(path, format!("field '{field}' is not valid UTF-8: {err}"))
        })
}

fn validate_metadata(
    path: &Path,
    metadata: PluginMetadata,
) -> Result<ValidatedPluginMetadata, String> {
    if metadata.abi_version != PLUGIN_ABI_VERSION && metadata.abi_version != PLUGIN_ABI_VERSION_V1 {
        return Err(plugin_metadata_error(
            path,
            format!(
                "ABI mismatch: expected {} or {}, got {}",
                PLUGIN_ABI_VERSION, PLUGIN_ABI_VERSION_V1, metadata.abi_version
            ),
        ));
    }

    let plugin_name = unsafe { cstr_to_string(metadata.plugin_name, "plugin_name", path) }?;
    validate_plugin_name(path, &plugin_name)?;

    let header_text = unsafe { cstr_to_string(metadata.header_text, "header_text", path) }?;
    validate_text_field(path, "metadata", "header_text", &header_text)?;

    Ok(ValidatedPluginMetadata {
        abi_version: metadata.abi_version,
        supports_only_diffline: metadata.supports_only_diffline,
        plugin_name,
        header_text,
    })
}

fn validate_plugin_name(path: &Path, plugin_name: &str) -> Result<(), String> {
    if plugin_name.is_empty() {
        return Err(plugin_metadata_error(path, "plugin_name must not be empty"));
    }
    if plugin_name.trim() != plugin_name {
        return Err(plugin_metadata_error(
            path,
            "plugin_name must not have leading or trailing whitespace",
        ));
    }
    if plugin_name
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return Err(plugin_metadata_error(
            path,
            format!(
                "plugin_name '{plugin_name}' must not contain whitespace or control characters"
            ),
        ));
    }
    Ok(())
}

fn validate_text_field(path: &Path, stage: &str, field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(plugin_error_at_stage(
            path,
            stage,
            format!("{field} must not be empty"),
        ));
    }
    if value.chars().any(|ch| ch.is_control()) {
        return Err(plugin_error_at_stage(
            path,
            stage,
            format!("{field} must not contain control characters"),
        ));
    }
    Ok(())
}

fn copy_plugin_response_bytes(
    bytes: PluginOwnedBytes,
    free_bytes: PluginFreeBytesFn,
    path: &Path,
) -> Result<Vec<u8>, String> {
    if bytes.ptr.is_null() {
        return Err(plugin_response_bytes_error(
            path,
            "plugin returned a null pointer",
        ));
    }
    if bytes.cap < bytes.len {
        return Err(plugin_response_bytes_error(
            path,
            format!(
                "invalid ownership metadata: len {} exceeds cap {}. free_bytes was skipped to avoid undefined behavior",
                bytes.len, bytes.cap
            ),
        ));
    }
    if bytes.len == 0 {
        unsafe {
            free_bytes(bytes);
        }
        return Err(plugin_response_bytes_error(
            path,
            "plugin returned an empty response",
        ));
    }
    if bytes.len > MAX_PLUGIN_RESPONSE_BYTES {
        unsafe {
            free_bytes(bytes);
        }
        return Err(plugin_response_bytes_error(
            path,
            format!(
                "response too large: {} bytes exceeds limit of {} bytes",
                bytes.len, MAX_PLUGIN_RESPONSE_BYTES
            ),
        ));
    }

    let raw = unsafe { slice::from_raw_parts(bytes.ptr, bytes.len).to_vec() };
    unsafe {
        free_bytes(bytes);
    }
    Ok(raw)
}

fn parse_plugin_response(
    bytes: PluginOwnedBytes,
    free_bytes: PluginFreeBytesFn,
    path: &Path,
) -> Result<ParsedPluginDiffResponse, String> {
    let json_bytes = copy_plugin_response_bytes(bytes, free_bytes, path)?;
    let json = String::from_utf8(json_bytes)
        .map_err(|err| plugin_response_error(path, format!("invalid UTF-8: {err}")))?;

    let response: RawPluginDiffResponse = serde_json::from_str(&json)
        .map_err(|err| plugin_response_error(path, format!("invalid JSON: {err}")))?;

    if response.schema_version != PLUGIN_RESPONSE_SCHEMA_V1
        && response.schema_version != PLUGIN_RESPONSE_SCHEMA_V2
        && response.schema_version != PLUGIN_RESPONSE_SCHEMA_V3
    {
        return Err(plugin_response_error(
            path,
            format!(
                "schema mismatch: expected {}, {}, or {}, got {}",
                PLUGIN_RESPONSE_SCHEMA_V1,
                PLUGIN_RESPONSE_SCHEMA_V2,
                PLUGIN_RESPONSE_SCHEMA_V3,
                response.schema_version
            ),
        ));
    }
    validate_text_field(path, "response", "header_text", &response.header_text)?;

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
            plugin_response_error(
                path,
                format!(
                    "line {} span {} has invalid style {} '{}': {}",
                    line_index + 1,
                    span_index + 1,
                    field,
                    value,
                    err
                ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    const TEST_PLUGIN_PATH: &str = "/tmp/test-plugin";

    unsafe extern "C" fn free_test_bytes(bytes: PluginOwnedBytes) {
        unsafe {
            hwatch_diffmode::drop_plugin_owned_bytes(bytes);
        }
    }

    unsafe extern "C" fn generate_invalid_json(_: PluginDiffRequest) -> PluginOwnedBytes {
        hwatch_diffmode::plugin_owned_bytes_from_vec(b"{".to_vec())
    }

    unsafe extern "C" fn generate_invalid_color(_: PluginDiffRequest) -> PluginOwnedBytes {
        hwatch_diffmode::plugin_owned_bytes_from_vec(
            br#"{"schema_version":3,"header_text":"Test","lines":[{"line_no":1,"spans":[{"text":"x","style":{"fg":"wat"}}]}]}"#
                .to_vec(),
        )
    }

    fn bytes_from_vec(bytes: Vec<u8>) -> PluginOwnedBytes {
        hwatch_diffmode::plugin_owned_bytes_from_vec(bytes)
    }

    fn test_path() -> &'static Path {
        Path::new(TEST_PLUGIN_PATH)
    }

    fn test_mode(generate: PluginGenerateFn) -> PluginDiffMode {
        PluginDiffMode {
            _library: None,
            generate,
            free_bytes: free_test_bytes,
            plugin_name: "test-mode".to_string(),
            header_text: "Test".to_string(),
            supports_only_diffline: true,
            options: DiffModeOptions::new(),
            plugin_path: test_path().to_path_buf(),
        }
    }

    #[test]
    fn validate_metadata_rejects_invalid_abi() {
        let plugin_name = CString::new("test-mode").unwrap();
        let header_text = CString::new("Test").unwrap();

        let error = validate_metadata(
            test_path(),
            PluginMetadata {
                abi_version: 99,
                supports_only_diffline: true,
                plugin_name: plugin_name.as_ptr(),
                header_text: header_text.as_ptr(),
            },
        )
        .unwrap_err();

        assert!(error.contains("ABI mismatch"));
    }

    #[test]
    fn validate_metadata_rejects_invalid_utf8_name() {
        let header_text = CString::new("Test").unwrap();
        let invalid_name = [0xff_u8, 0x00];

        let error = validate_metadata(
            test_path(),
            PluginMetadata {
                abi_version: PLUGIN_ABI_VERSION,
                supports_only_diffline: true,
                plugin_name: invalid_name.as_ptr().cast(),
                header_text: header_text.as_ptr(),
            },
        )
        .unwrap_err();

        assert!(error.contains("plugin_name"));
        assert!(error.contains("UTF-8"));
    }

    #[test]
    fn validate_metadata_rejects_null_header_text() {
        let plugin_name = CString::new("test-mode").unwrap();

        let error = validate_metadata(
            test_path(),
            PluginMetadata {
                abi_version: PLUGIN_ABI_VERSION,
                supports_only_diffline: true,
                plugin_name: plugin_name.as_ptr(),
                header_text: std::ptr::null(),
            },
        )
        .unwrap_err();

        assert!(error.contains("header_text"));
        assert!(error.contains("null"));
    }

    #[test]
    fn validate_metadata_rejects_empty_plugin_name() {
        let plugin_name = CString::new("").unwrap();
        let header_text = CString::new("Test").unwrap();

        let error = validate_metadata(
            test_path(),
            PluginMetadata {
                abi_version: PLUGIN_ABI_VERSION,
                supports_only_diffline: true,
                plugin_name: plugin_name.as_ptr(),
                header_text: header_text.as_ptr(),
            },
        )
        .unwrap_err();

        assert!(error.contains("plugin_name must not be empty"));
    }

    #[test]
    fn validate_metadata_rejects_plugin_name_with_whitespace() {
        let plugin_name = CString::new("test mode").unwrap();
        let header_text = CString::new("Test").unwrap();

        let error = validate_metadata(
            test_path(),
            PluginMetadata {
                abi_version: PLUGIN_ABI_VERSION,
                supports_only_diffline: true,
                plugin_name: plugin_name.as_ptr(),
                header_text: header_text.as_ptr(),
            },
        )
        .unwrap_err();

        assert!(error.contains("must not contain whitespace"));
    }

    #[test]
    fn parse_plugin_response_rejects_invalid_utf8() {
        let error = parse_plugin_response(
            bytes_from_vec(vec![0xff, 0x00]),
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("response: invalid UTF-8"));
    }

    #[test]
    fn parse_plugin_response_rejects_empty_response() {
        let error = parse_plugin_response(
            bytes_from_vec(Vec::new()),
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("empty response"));
    }

    #[test]
    fn parse_plugin_response_rejects_null_pointer() {
        let error = parse_plugin_response(
            PluginOwnedBytes {
                ptr: std::ptr::null_mut(),
                len: 1,
                cap: 1,
            },
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("null pointer"));
    }

    #[test]
    fn parse_plugin_response_rejects_invalid_json() {
        let error =
            parse_plugin_response(bytes_from_vec(b"{".to_vec()), free_test_bytes, test_path())
                .unwrap_err();

        assert!(error.contains("response: invalid JSON"));
    }

    #[test]
    fn parse_plugin_response_rejects_empty_header_text() {
        let error = parse_plugin_response(
            bytes_from_vec(br#"{"schema_version":3,"header_text":"   ","lines":[]}"#.to_vec()),
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("header_text must not be empty"));
    }

    #[test]
    fn parse_plugin_response_rejects_unsupported_schema() {
        let error = parse_plugin_response(
            bytes_from_vec(br#"{"schema_version":999,"header_text":"Test","lines":[]}"#.to_vec()),
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("schema mismatch"));
    }

    #[test]
    fn parse_plugin_response_rejects_invalid_style_color() {
        let error = parse_plugin_response(
            bytes_from_vec(
                br#"{"schema_version":3,"header_text":"Test","lines":[{"line_no":1,"spans":[{"text":"x","style":{"fg":"wat"}}]}]}"#
                    .to_vec(),
            ),
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("invalid style fg 'wat'"));
    }

    #[test]
    fn parse_plugin_response_rejects_invalid_owned_bytes_layout() {
        let error = parse_plugin_response(
            PluginOwnedBytes {
                ptr: std::ptr::NonNull::<u8>::dangling().as_ptr(),
                len: 8,
                cap: 4,
            },
            free_test_bytes,
            test_path(),
        )
        .unwrap_err();

        assert!(error.contains("len 8 exceeds cap 4"));
        assert!(error.contains("free_bytes was skipped"));
    }

    #[test]
    fn batch_diff_falls_back_to_error_line_on_invalid_json() {
        let mut mode = test_mode(PluginGenerateFn::V2(generate_invalid_json));

        let lines = mode.generate_batch_diff("dest", "src");

        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("error:"));
        assert!(lines[0].contains("invalid JSON"));
    }

    #[test]
    fn watch_diff_falls_back_to_error_line_on_invalid_style() {
        let mut mode = test_mode(PluginGenerateFn::V2(generate_invalid_color));

        let lines = mode.generate_watch_diff("dest", "src");

        assert_eq!(lines.len(), 1);
        let text = lines[0]
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(text.contains("error:"));
        assert!(text.contains("invalid style fg 'wat'"));
    }
}
