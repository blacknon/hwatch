// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use std::path::{Path, PathBuf};

use hwatch_diffmode::{
    DiffMode, DiffModeOptions, PluginDiffRequest, PluginDiffRequestV1, PluginMetadata,
    PluginOwnedBytes, PluginSlice, PLUGIN_ABI_VERSION, PLUGIN_ABI_VERSION_V1,
    PLUGIN_OUTPUT_BATCH, PLUGIN_OUTPUT_WATCH,
};
use libloading::{Library, Symbol};
use tui::prelude::Line;

#[path = "plugin_diffmode_render.rs"]
mod render;
#[path = "plugin_diffmode_response.rs"]
mod response;
#[path = "plugin_diffmode_types.rs"]
mod types;
#[path = "plugin_diffmode_validation.rs"]
mod validation;

use self::render::{render_batch_line, render_watch_line};
use self::response::parse_plugin_response;
use self::types::ParsedPluginDiffResponse;
use self::validation::validate_metadata;

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
        let error = parse_plugin_response(bytes_from_vec(Vec::new()), free_test_bytes, test_path())
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
