// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{
    plugin_error_at_stage, plugin_metadata_error, types::ValidatedPluginMetadata, PluginMetadata,
};
use hwatch_diffmode::{PLUGIN_ABI_VERSION, PLUGIN_ABI_VERSION_V1};
use std::ffi::CStr;
use std::path::Path;

pub(super) unsafe fn cstr_to_string(
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

pub(super) fn validate_metadata(
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

pub(super) fn validate_text_field(
    path: &Path,
    stage: &str,
    field: &str,
    value: &str,
) -> Result<(), String> {
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
