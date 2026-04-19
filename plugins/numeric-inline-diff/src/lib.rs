use std::slice;
use std::str;

use hwatch_ansi as ansi;
use hwatch_diffmode::{
    PluginDiffRequest as HwatchDiffRequest, PluginMetadata as HwatchPluginMetadata,
    PluginOwnedBytes as HwatchOwnedBytes, PluginSlice as HwatchSlice, PLUGIN_ABI_VERSION,
};
use similar::{ChangeTag, TextDiff};

static PLUGIN_NAME: &[u8] = b"numeric-inline-diff\0";
static HEADER_TEXT: &[u8] = b"NumInline\0";
const RESPONSE_SCHEMA_VERSION: u32 = 3;
const WATCH_LINE_ADD: &str = "green";
const WATCH_LINE_REM: &str = "red";

#[derive(Clone, Debug, PartialEq)]
struct NumericToken {
    raw: String,
    value: f64,
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct RenderedLine {
    line_no: Option<usize>,
    diff_type: &'static str,
    spans: Vec<RenderedSpan>,
}

#[derive(Clone, Debug, PartialEq)]
struct RenderedSpan {
    text: String,
    fg: Option<&'static str>,
    reversed: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ChangedLine {
    line_no: Option<usize>,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
struct NumericMatch {
    before: NumericToken,
    after: NumericToken,
}

#[derive(Clone, Copy)]
enum RenderKind {
    Equal,
    Remove,
    Insert,
}

#[no_mangle]
pub extern "C" fn hwatch_diffmode_metadata() -> HwatchPluginMetadata {
    HwatchPluginMetadata {
        abi_version: PLUGIN_ABI_VERSION,
        supports_only_diffline: true,
        plugin_name: PLUGIN_NAME.as_ptr().cast(),
        header_text: HEADER_TEXT.as_ptr().cast(),
    }
}

#[no_mangle]
pub extern "C" fn hwatch_diffmode_generate(req: HwatchDiffRequest) -> HwatchOwnedBytes {
    let dest = unsafe { slice_to_str(req.dest) }.unwrap_or_default();
    let src = unsafe { slice_to_str(req.src) }.unwrap_or_default();

    let lines =
        generate_numeric_inline_diff(dest, src, req.line_number, req.only_diffline, req.color);
    let header_text = if req.only_diffline {
        "NumInline(Only)"
    } else {
        "NumInline"
    };

    let json = render_json_response(header_text, &lines);
    into_owned_bytes(json.into_bytes())
}

#[no_mangle]
pub extern "C" fn hwatch_diffmode_free_bytes(bytes: HwatchOwnedBytes) {
    if bytes.ptr.is_null() || bytes.cap == 0 {
        return;
    }

    unsafe {
        drop(Vec::from_raw_parts(bytes.ptr, bytes.len, bytes.cap));
    }
}

unsafe fn slice_to_str(input: HwatchSlice) -> Option<&'static str> {
    if input.ptr.is_null() {
        return None;
    }

    let bytes = slice::from_raw_parts(input.ptr, input.len);
    str::from_utf8(bytes).ok()
}

fn generate_numeric_inline_diff(
    dest: &str,
    src: &str,
    _line_number: bool,
    only_diffline: bool,
    color: bool,
) -> Vec<RenderedLine> {
    let diff = TextDiff::from_lines(src, dest);
    let changes: Vec<_> = diff.iter_all_changes().collect();

    let mut rendered = Vec::new();
    let mut index = 0;
    while index < changes.len() {
        let change = &changes[index];

        if change.tag() != ChangeTag::Equal {
            let mut deletes = Vec::new();
            let mut inserts = Vec::new();

            while index < changes.len() && changes[index].tag() != ChangeTag::Equal {
                let current = &changes[index];
                match current.tag() {
                    ChangeTag::Delete => deletes.push(ChangedLine {
                        line_no: current.old_index().map(|value| value + 1),
                        text: strip_changed_line(current.to_string()),
                    }),
                    ChangeTag::Insert => inserts.push(ChangedLine {
                        line_no: current.new_index().map(|value| value + 1),
                        text: strip_changed_line(current.to_string()),
                    }),
                    ChangeTag::Equal => {}
                }
                index += 1;
            }

            render_changed_block(
                &mut rendered,
                deletes,
                inserts,
            );
            continue;
        }

        if !only_diffline {
            rendered.push(render_line(
                change.old_index().map(|value| value + 1),
                "   ",
                vec![RenderedSpan {
                    text: normalize_equal_line(change.to_string(), color),
                    fg: None,
                    reversed: false,
                }],
                RenderKind::Equal,
            ));
        }

        index += 1;
    }

    rendered
}

fn render_changed_block(
    rendered: &mut Vec<RenderedLine>,
    deletes: Vec<ChangedLine>,
    inserts: Vec<ChangedLine>,
) {
    let mut used_inserts = vec![false; inserts.len()];

    for delete in &deletes {
        if let Some((insert, matches)) =
            find_matching_insert(&delete.text, &inserts, &mut used_inserts)
        {
            rendered.push(render_line(
                delete.line_no,
                "-  ",
                build_inline_spans(&delete.text, &matches, true),
                RenderKind::Remove,
            ));
            rendered.push(render_line(
                insert.line_no,
                "+  ",
                build_inline_spans(&insert.text, &matches, false),
                RenderKind::Insert,
            ));
        } else {
            rendered.push(render_plain_changed_line(
                delete.line_no,
                "-  ",
                delete.text.clone(),
                RenderKind::Remove,
            ));
        }
    }

    for (index, insert) in inserts.into_iter().enumerate() {
        if !used_inserts[index] {
            rendered.push(render_plain_changed_line(
                insert.line_no,
                "+  ",
                insert.text,
                RenderKind::Insert,
            ));
        }
    }
}

fn find_matching_insert<'a>(
    before: &str,
    inserts: &'a [ChangedLine],
    used_inserts: &mut [bool],
) -> Option<(&'a ChangedLine, Vec<NumericMatch>)> {
    for (index, insert) in inserts.iter().enumerate() {
        if used_inserts[index] {
            continue;
        }

        if let Some(matches) = match_numeric_tokens(before, &insert.text) {
            used_inserts[index] = true;
            return Some((insert, matches));
        }
    }

    None
}

fn match_numeric_tokens(before: &str, after: &str) -> Option<Vec<NumericMatch>> {
    let (before_numbers, before_skeleton) = extract_numeric_tokens(before);
    let (after_numbers, after_skeleton) = extract_numeric_tokens(after);

    if before_numbers.is_empty()
        || before_numbers.len() != after_numbers.len()
        || before_skeleton != after_skeleton
    {
        return None;
    }

    let mut matches = Vec::new();
    for (before_token, after_token) in before_numbers.iter().zip(after_numbers.iter()) {
        if before_token.raw == after_token.raw {
            continue;
        }

        matches.push(NumericMatch {
            before: before_token.clone(),
            after: after_token.clone(),
        });
    }

    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

fn build_inline_spans(source: &str, matches: &[NumericMatch], use_before: bool) -> Vec<RenderedSpan> {
    let chars: Vec<char> = source.chars().collect();
    let mut spans = Vec::new();
    let mut cursor = 0;

    for numeric_match in matches {
        let token = if use_before {
            &numeric_match.before
        } else {
            &numeric_match.after
        };

        if token.start > cursor {
            spans.push(RenderedSpan {
                text: chars[cursor..token.start].iter().collect(),
                fg: Some(base_text_color(use_before)),
                reversed: false,
            });
        }

        spans.push(RenderedSpan {
            text: chars[token.start..token.end].iter().collect(),
            fg: Some(delta_color(numeric_match.after.value - numeric_match.before.value)),
            reversed: true,
        });
        cursor = token.end;
    }

    if cursor < chars.len() {
        spans.push(RenderedSpan {
            text: chars[cursor..].iter().collect(),
            fg: Some(base_text_color(use_before)),
            reversed: false,
        });
    }

    spans
}

fn render_plain_changed_line(
    line_no: Option<usize>,
    marker: &'static str,
    text: String,
    kind: RenderKind,
) -> RenderedLine {
    render_line(
        line_no,
        marker,
        vec![RenderedSpan {
            text,
            fg: Some(base_text_color(matches!(kind, RenderKind::Remove))),
            reversed: false,
        }],
        kind,
    )
}

fn render_line(
    line_no: Option<usize>,
    marker: &'static str,
    body_spans: Vec<RenderedSpan>,
    kind: RenderKind,
) -> RenderedLine {
    let mut spans = Vec::new();

    spans.push(RenderedSpan {
        text: marker.to_string(),
        fg: marker_color(kind),
        reversed: false,
    });
    spans.extend(body_spans);

    RenderedLine {
        line_no,
        diff_type: diff_type_name(kind),
        spans,
    }
}

fn trim_line_end(mut line: String) -> String {
    if line.ends_with('\n') {
        line.pop();
    }
    line
}

fn normalize_equal_line(line: String, color: bool) -> String {
    let line = trim_line_end(line);
    if color {
        line
    } else {
        ansi::get_ansi_strip_str(&line)
    }
}

fn strip_changed_line(line: String) -> String {
    ansi::get_ansi_strip_str(&trim_line_end(line))
}

fn extract_numeric_tokens(input: &str) -> (Vec<NumericToken>, String) {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    let mut skeleton = String::new();
    let mut tokens = Vec::new();
    let mut last_was_whitespace = false;

    while index < chars.len() {
        if let Some((end, raw)) = parse_number_token(&chars, index) {
            if let Some(value) = raw.parse::<f64>().ok() {
                tokens.push(NumericToken {
                    raw: raw.clone(),
                    value,
                    start: index,
                    end,
                });
                skeleton.push('#');
                index = end;
                last_was_whitespace = false;
                continue;
            }
        }

        let ch = chars[index];
        if ch.is_whitespace() {
            if !last_was_whitespace {
                skeleton.push(' ');
                last_was_whitespace = true;
            }
        } else {
            skeleton.push(ch);
            last_was_whitespace = false;
        }
        index += 1;
    }

    (tokens, skeleton)
}

fn parse_number_token(chars: &[char], start: usize) -> Option<(usize, String)> {
    let mut index = start;
    let first = *chars.get(index)?;

    if first == '+' || first == '-' {
        let next = *chars.get(index + 1)?;
        if !next.is_ascii_digit() {
            return None;
        }

        if start > 0 {
            let prev = chars[start - 1];
            if prev.is_ascii_alphanumeric() || prev == '.' {
                return None;
            }
        }

        index += 1;
    } else if !first.is_ascii_digit() {
        return None;
    }

    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut token = String::new();

    while let Some(ch) = chars.get(index) {
        if ch.is_ascii_digit() {
            seen_digit = true;
            token.push(*ch);
            index += 1;
            continue;
        }

        if *ch == '.' && !seen_dot {
            seen_dot = true;
            token.push(*ch);
            index += 1;
            continue;
        }

        break;
    }

    if start < chars.len() && (chars[start] == '+' || chars[start] == '-') {
        token.insert(0, chars[start]);
    }

    if !seen_digit || token.ends_with('.') {
        return None;
    }

    Some((index, token))
}

fn marker_color(kind: RenderKind) -> Option<&'static str> {
    match kind {
        RenderKind::Equal => None,
        RenderKind::Remove => Some(WATCH_LINE_REM),
        RenderKind::Insert => Some(WATCH_LINE_ADD),
    }
}

fn base_text_color(is_before: bool) -> &'static str {
    if is_before {
        "red"
    } else {
        "green"
    }
}

fn diff_type_name(kind: RenderKind) -> &'static str {
    match kind {
        RenderKind::Equal => "same",
        RenderKind::Remove => "rem",
        RenderKind::Insert => "add",
    }
}

fn delta_color(delta: f64) -> &'static str {
    if delta > 0.0 {
        "green"
    } else {
        "red"
    }
}

fn render_json_response(header_text: &str, lines: &[RenderedLine]) -> String {
    let mut json = format!(
        "{{\"schema_version\":{},\"header_text\":\"",
        RESPONSE_SCHEMA_VERSION
    );
    json.push_str(&escape_json(header_text));
    json.push_str("\",\"lines\":[");

    for (line_index, line) in lines.iter().enumerate() {
        if line_index > 0 {
            json.push(',');
        }

        json.push_str("{");
        if let Some(line_no) = line.line_no {
            json.push_str("\"line_no\":");
            json.push_str(&line_no.to_string());
            json.push(',');
        }
        json.push_str("\"diff_type\":\"");
        json.push_str(line.diff_type);
        json.push_str("\",\"spans\":[");
        for (span_index, span) in line.spans.iter().enumerate() {
            if span_index > 0 {
                json.push(',');
            }

            json.push_str("{\"text\":\"");
            json.push_str(&escape_json(&span.text));
            json.push('"');

            if span.fg.is_some() || span.reversed {
                json.push_str(",\"style\":{");
                let mut has_prev = false;
                if let Some(fg) = span.fg {
                    json.push_str("\"fg\":\"");
                    json.push_str(&escape_json(fg));
                    json.push('"');
                    has_prev = true;
                }
                if span.reversed {
                    if has_prev {
                        json.push(',');
                    }
                    json.push_str("\"reversed\":true");
                }
                json.push('}');
            }

            json.push('}');
        }
        json.push_str("]}");
    }

    json.push_str("]}");
    json
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }

    escaped
}

fn into_owned_bytes(mut bytes: Vec<u8>) -> HwatchOwnedBytes {
    let output = HwatchOwnedBytes {
        ptr: bytes.as_mut_ptr(),
        len: bytes.len(),
        cap: bytes.capacity(),
    };
    std::mem::forget(bytes);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_inline_numeric_changes() {
        let actual =
            generate_numeric_inline_diff("count=15 used=92\n", "count=10 used=100\n", true, false, false);

        assert_eq!(actual.len(), 2);
        assert_eq!(actual[0].line_no, Some(1));
        assert_eq!(actual[0].diff_type, "rem");
        assert_eq!(actual[0].spans[0].text, "-  ");
        assert_eq!(actual[0].spans[0].fg, Some(WATCH_LINE_REM));
        assert_eq!(actual[0].spans[2].text, "10");
        assert_eq!(actual[0].spans[2].fg, Some("green"));
        assert!(actual[0].spans[2].reversed);
        assert_eq!(actual[0].spans[4].text, "100");
        assert_eq!(actual[0].spans[4].fg, Some("red"));
        assert!(actual[0].spans[4].reversed);
        assert_eq!(actual[1].diff_type, "add");
        assert_eq!(actual[1].spans[0].fg, Some(WATCH_LINE_ADD));
        assert_eq!(actual[1].spans[2].text, "15");
        assert_eq!(actual[1].spans[2].fg, Some("green"));
        assert!(actual[1].spans[2].reversed);
    }

    #[test]
    fn ignores_whitespace_alignment_differences() {
        let actual = match_numeric_tokens(
            "-rw-r--r--  1 blacknon  staff   999 Apr 17 18:37 file.txt",
            "-rw-r--r--  1 blacknon  staff  1000 Apr 17 18:37 file.txt",
        );

        assert!(actual.is_some());
    }

    #[test]
    fn strips_ansi_from_changed_lines_even_when_color_is_enabled() {
        let actual = generate_numeric_inline_diff(
            "\u{1b}[32mvalue=2\u{1b}[0m\n",
            "\u{1b}[31mvalue=1\u{1b}[0m\n",
            true,
            false,
            true,
        );

        assert_eq!(actual[0].spans[2].text, "value=");
        assert_eq!(actual[0].spans[3].text, "1");
        assert_eq!(actual[1].spans[2].text, "value=");
        assert_eq!(actual[1].spans[3].text, "2");
    }

    #[test]
    fn preserves_ansi_on_equal_lines_only_when_color_is_enabled() {
        let actual = generate_numeric_inline_diff(
            "\u{1b}[32msame\u{1b}[0m\n",
            "\u{1b}[32msame\u{1b}[0m\n",
            false,
            false,
            true,
        );
        assert_eq!(actual[0].spans[0].text, "\u{1b}[32msame\u{1b}[0m");

        let stripped = generate_numeric_inline_diff(
            "\u{1b}[32msame\u{1b}[0m\n",
            "\u{1b}[32msame\u{1b}[0m\n",
            false,
            false,
            false,
        );
        assert_eq!(stripped[0].spans[0].text, "same");
    }
}
