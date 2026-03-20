// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::slice;

use hwatch_diffmode::{
    DiffMode, DiffModeOptions, PluginDiffRequest, PluginMetadata, PluginOwnedBytes, PluginSlice,
    PLUGIN_ABI_VERSION, PLUGIN_OUTPUT_BATCH, PLUGIN_OUTPUT_WATCH,
};
use libloading::{Library, Symbol};
use serde::Deserialize;
use tui::prelude::Line;

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

#[derive(Deserialize)]
struct PluginDiffResponse {
    schema_version: u32,
    header_text: String,
    lines: Vec<String>,
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

    fn invoke(&mut self, output_kind: u32, dest: &str, src: &str) -> Result<Vec<String>, String> {
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
        self.header_text = response.header_text;
        Ok(response.lines)
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
            Ok(lines) => lines.into_iter().map(Line::from).collect(),
            Err(err) => self.error_lines(err).into_iter().map(Line::from).collect(),
        }
    }

    fn generate_batch_diff(&mut self, dest: &str, src: &str) -> Vec<String> {
        match self.invoke(PLUGIN_OUTPUT_BATCH, dest, src) {
            Ok(lines) => lines,
            Err(err) => self.error_lines(err),
        }
    }

    fn get_header_text(&self) -> String {
        self.header_text.clone()
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
) -> Result<PluginDiffResponse, String> {
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

    let response: PluginDiffResponse = serde_json::from_str(&json)
        .map_err(|err| format!("plugin '{}' returned invalid JSON: {err}", path.display()))?;

    if response.schema_version != PLUGIN_ABI_VERSION {
        return Err(format!(
            "plugin '{}' response schema mismatch: expected {}, got {}",
            path.display(),
            PLUGIN_ABI_VERSION,
            response.schema_version
        ));
    }

    Ok(response)
}
