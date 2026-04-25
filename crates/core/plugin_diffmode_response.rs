// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{
    plugin_response_bytes_error, plugin_response_error, validation::validate_text_field,
    types::{
        ParsedPluginDiffResponse, PluginLine, PluginStyledLine, RawPluginDiffResponse,
        RawPluginLine,
    },
    PluginFreeBytesFn, PluginOwnedBytes, MAX_PLUGIN_RESPONSE_BYTES, PLUGIN_RESPONSE_SCHEMA_V1,
    PLUGIN_RESPONSE_SCHEMA_V2, PLUGIN_RESPONSE_SCHEMA_V3,
};
use crate::common::parse_ansi_color;
use std::path::Path;
use std::slice;

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

pub(super) fn parse_plugin_response(
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
