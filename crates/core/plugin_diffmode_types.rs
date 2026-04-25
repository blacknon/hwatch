// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use hwatch_diffmode::DifferenceType;
use serde::Deserialize;

#[derive(Debug)]
pub(super) struct ParsedPluginDiffResponse {
    pub(super) header_text: String,
    pub(super) lines: Vec<PluginLine>,
    pub(super) line_number_width: usize,
    pub(super) use_core_gutter: bool,
}

#[derive(Deserialize)]
pub(super) struct RawPluginDiffResponse {
    pub(super) schema_version: u32,
    pub(super) header_text: String,
    pub(super) lines: Vec<RawPluginLine>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum RawPluginLine {
    Plain(String),
    Styled(PluginStyledLine),
}

#[derive(Debug)]
pub(super) enum PluginLine {
    Plain(String),
    Styled(PluginStyledLine),
}

#[derive(Debug, Deserialize)]
pub(super) struct PluginStyledLine {
    #[serde(default)]
    pub(super) line_no: Option<usize>,
    #[serde(default)]
    pub(super) diff_type: Option<PluginDiffType>,
    #[serde(default)]
    pub(super) gutter: Option<PluginGutterSpec>,
    pub(super) spans: Vec<PluginStyledSpan>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PluginStyledSpan {
    pub(super) text: String,
    #[serde(default)]
    pub(super) style: PluginStyleSpec,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct PluginStyleSpec {
    pub(super) fg: Option<String>,
    pub(super) bg: Option<String>,
    #[serde(default)]
    pub(super) bold: bool,
    #[serde(default)]
    pub(super) dim: bool,
    #[serde(default)]
    pub(super) italic: bool,
    #[serde(default)]
    pub(super) underlined: bool,
    #[serde(default)]
    pub(super) reversed: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum PluginDiffType {
    Same,
    Add,
    Rem,
}

impl PluginDiffType {
    pub(super) fn as_difference_type(self) -> DifferenceType {
        match self {
            Self::Same => DifferenceType::Same,
            Self::Add => DifferenceType::Add,
            Self::Rem => DifferenceType::Rem,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct PluginGutterSpec {
    #[serde(default)]
    pub(super) text: Option<String>,
    #[serde(default)]
    pub(super) style: PluginStyleSpec,
}

#[derive(Debug)]
pub(super) struct ValidatedPluginMetadata {
    pub(super) abi_version: u32,
    pub(super) supports_only_diffline: bool,
    pub(super) plugin_name: String,
    pub(super) header_text: String,
}
