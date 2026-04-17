use std::slice;
use std::str;

use hwatch_ansi as ansi;
use hwatch_diffmode::{
    PluginDiffRequest as HwatchDiffRequest, PluginMetadata as HwatchPluginMetadata,
    PluginOwnedBytes as HwatchOwnedBytes, PluginSlice as HwatchSlice, PLUGIN_ABI_VERSION,
};
use similar::{ChangeTag, TextDiff};

static PLUGIN_NAME: &[u8] = b"numeric-diff\0";
static HEADER_TEXT: &[u8] = b"NumDiff\0";
const RESPONSE_SCHEMA_VERSION: u32 = 2;
const WATCH_LINE_NUMBER_DEFAULT: &str = "darkgray";
const WATCH_LINE_NUMBER_ADD: &str = "56,119,120";
const WATCH_LINE_NUMBER_REM: &str = "118,0,0";
const WATCH_LINE_ADD: &str = "green";
const WATCH_LINE_REM: &str = "red";
const NUMERIC_ANNOTATION: &str = "184,134,11";
const NUMERIC_ANNOTATION_CARET: &str = "yellow";

#[derive(Clone, Debug, PartialEq)]
struct NumericToken {
    raw: String,
    value: f64,
    start: usize,
}

#[derive(Clone)]
struct RenderedLine {
    spans: Vec<RenderedSpan>,
}

#[derive(Clone)]
struct RenderedSpan {
    text: String,
    fg: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
struct NumericDelta {
    start: usize,
    before_delta: String,
    after_delta: String,
}

#[derive(Clone)]
struct ChangedLine {
    line_no: Option<usize>,
    text: String,
}

#[derive(Clone, Copy)]
enum RenderKind {
    Equal,
    Remove,
    Insert,
    RemoveAnnotation,
    InsertAnnotation,
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

    let lines = generate_numeric_diff(dest, src, req.line_number, req.only_diffline, req.color);
    let header_text = if req.only_diffline {
        "NumDiff(Only)"
    } else {
        "NumDiff"
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

fn generate_numeric_diff(
    dest: &str,
    src: &str,
    line_number: bool,
    only_diffline: bool,
    color: bool,
) -> Vec<RenderedLine> {
    let diff = TextDiff::from_lines(src, dest);
    let changes: Vec<_> = diff.iter_all_changes().collect();
    let line_width = diff
        .old_slices()
        .len()
        .max(diff.new_slices().len())
        .to_string()
        .len();

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
                line_number,
                line_width,
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
                }],
                line_number,
                line_width,
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
    line_number: bool,
    line_width: usize,
) {
    let mut used_inserts = vec![false; inserts.len()];

    for delete in &deletes {
        rendered.push(render_plain_changed_line(
            delete.line_no,
            "-  ",
            delete.text.clone(),
            line_number,
            line_width,
            RenderKind::Remove,
        ));

        if let Some((insert, deltas)) = find_matching_insert(&delete.text, &inserts, &mut used_inserts) {
            rendered.push(render_line(
                delete.line_no,
                "*- ",
                build_annotation_spans(&delete.text, &deltas, true),
                line_number,
                line_width,
                RenderKind::RemoveAnnotation,
            ));
            rendered.push(render_plain_changed_line(
                insert.line_no,
                "+  ",
                insert.text.clone(),
                line_number,
                line_width,
                RenderKind::Insert,
            ));
            rendered.push(render_line(
                insert.line_no,
                "*+ ",
                build_annotation_spans(&insert.text, &deltas, false),
                line_number,
                line_width,
                RenderKind::InsertAnnotation,
            ));
        }
    }

    for (index, insert) in inserts.into_iter().enumerate() {
        if !used_inserts[index] {
            rendered.push(render_plain_changed_line(
                insert.line_no,
                "+  ",
                insert.text,
                line_number,
                line_width,
                RenderKind::Insert,
            ));
        }
    }
}

fn find_matching_insert<'a>(
    before: &str,
    inserts: &'a [ChangedLine],
    used_inserts: &mut [bool],
) -> Option<(&'a ChangedLine, Vec<NumericDelta>)> {
    for (index, insert) in inserts.iter().enumerate() {
        if used_inserts[index] {
            continue;
        }

        if let Some(deltas) = describe_numeric_delta(before, &insert.text) {
            used_inserts[index] = true;
            return Some((insert, deltas));
        }
    }

    None
}

fn render_plain_changed_line(
    line_no: Option<usize>,
    marker: &'static str,
    text: String,
    line_number: bool,
    line_width: usize,
    kind: RenderKind,
) -> RenderedLine {
    render_line(
        line_no,
        marker,
        vec![RenderedSpan {
            text,
            fg: match kind {
                RenderKind::Remove => Some("red"),
                RenderKind::Insert => Some("green"),
                _ => None,
            },
        }],
        line_number,
        line_width,
        kind,
    )
}

fn render_line(
    line_no: Option<usize>,
    marker: &'static str,
    body_spans: Vec<RenderedSpan>,
    line_number: bool,
    line_width: usize,
    kind: RenderKind,
) -> RenderedLine {
    let mut spans = Vec::new();

    if line_number {
        spans.push(RenderedSpan {
            text: match line_no {
                Some(number) => format!("{number:>line_width$} | "),
                None => format!("{:>line_width$} | ", ""),
            },
            fg: line_number_color(kind),
        });
    }

    spans.push(RenderedSpan {
        text: marker.to_string(),
        fg: marker_color(kind),
    });
    spans.extend(body_spans);

    RenderedLine { spans }
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

fn describe_numeric_delta(before: &str, after: &str) -> Option<Vec<NumericDelta>> {
    let (before_numbers, before_skeleton) = extract_numeric_tokens(before);
    let (after_numbers, after_skeleton) = extract_numeric_tokens(after);

    if before_numbers.is_empty()
        || before_numbers.len() != after_numbers.len()
        || before_skeleton != after_skeleton
    {
        return None;
    }

    let mut deltas = Vec::new();
    for (before_token, after_token) in before_numbers.iter().zip(after_numbers.iter()) {
        if before_token.raw == after_token.raw {
            continue;
        }

        deltas.push(NumericDelta {
            start: before_token.start,
            before_delta: format_signed_number(before_token.value - after_token.value),
            after_delta: format_signed_number(after_token.value - before_token.value),
        });
    }

    if deltas.is_empty() {
        None
    } else {
        Some(deltas)
    }
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

fn build_annotation_spans(source: &str, deltas: &[NumericDelta], use_before: bool) -> Vec<RenderedSpan> {
    let base_len = source.chars().count();
    let mut chars = vec![' '; base_len];
    let mut overlays = Vec::new();

    for delta in deltas {
        let text = if use_before {
            format!("^{}", delta.before_delta)
        } else {
            format!("^{}", delta.after_delta)
        };

        let start = write_overlay(&mut chars, delta.start, &text);
        overlays.push((start, text));
    }

    let plain = chars.into_iter().collect::<String>().trim_end().to_string();
    let mut spans = Vec::new();
    let mut char_index = 0;

    for (start, token) in overlays {
        if start > char_index {
            spans.push(RenderedSpan {
                text: plain.chars().skip(char_index).take(start - char_index).collect(),
                fg: Some(NUMERIC_ANNOTATION),
            });
        }

        spans.push(RenderedSpan {
            text: "^".to_string(),
            fg: Some(NUMERIC_ANNOTATION_CARET),
        });
        spans.push(RenderedSpan {
            text: token.chars().skip(1).collect(),
            fg: delta_color(&token),
        });
        char_index = start + token.chars().count();
    }

    let plain_len = plain.chars().count();
    if char_index < plain_len {
        spans.push(RenderedSpan {
            text: plain.chars().skip(char_index).collect(),
            fg: Some(NUMERIC_ANNOTATION),
        });
    }

    spans
}

fn write_overlay(buffer: &mut Vec<char>, mut start: usize, text: &str) -> usize {
    let overlay: Vec<char> = text.chars().collect();

    while has_overlap(buffer, start, overlay.len()) {
        start += 1;
    }

    let required_len = start + overlay.len();
    if buffer.len() < required_len {
        buffer.resize(required_len, ' ');
    }

    for (offset, ch) in overlay.iter().enumerate() {
        buffer[start + offset] = *ch;
    }

    start
}

fn has_overlap(buffer: &[char], start: usize, len: usize) -> bool {
    let end = start.saturating_add(len).min(buffer.len());
    buffer[start..end].iter().any(|ch| *ch != ' ')
}

fn format_signed_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{:+}", value as i64)
    } else {
        let mut text = format!("{:+.6}", value);
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

fn marker_color(kind: RenderKind) -> Option<&'static str> {
    match kind {
        RenderKind::Equal => None,
        RenderKind::Remove => Some(WATCH_LINE_REM),
        RenderKind::Insert => Some(WATCH_LINE_ADD),
        RenderKind::RemoveAnnotation | RenderKind::InsertAnnotation => Some(NUMERIC_ANNOTATION),
    }
}

fn line_number_color(kind: RenderKind) -> Option<&'static str> {
    match kind {
        RenderKind::Equal => Some(WATCH_LINE_NUMBER_DEFAULT),
        RenderKind::Remove => Some(WATCH_LINE_NUMBER_REM),
        RenderKind::Insert => Some(WATCH_LINE_NUMBER_ADD),
        RenderKind::RemoveAnnotation | RenderKind::InsertAnnotation => Some(NUMERIC_ANNOTATION),
    }
}

fn delta_color(delta: &str) -> Option<&'static str> {
    match delta.chars().nth(1) {
        Some('+') => Some("cyan"),
        Some('-') => Some("magenta"),
        _ => Some("184,134,11"),
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

        json.push_str("{\"spans\":[");
        for (span_index, span) in line.spans.iter().enumerate() {
            if span_index > 0 {
                json.push(',');
            }

            json.push_str("{\"text\":\"");
            json.push_str(&escape_json(&span.text));
            json.push('"');

            if let Some(fg) = span.fg {
                json.push_str(",\"style\":{\"fg\":\"");
                json.push_str(&escape_json(fg));
                json.push_str("\"}");
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
    fn inserts_numeric_annotation_lines_for_replaced_line() {
        let actual =
            generate_numeric_diff("retry=5 timeout=92\n", "retry=3 timeout=100\n", true, false, false);

        assert_eq!(actual.len(), 4);
        assert_eq!(actual[0].spans[0].text, "1 | ");
        assert_eq!(actual[0].spans[1].text, "-  ");
        assert_eq!(actual[1].spans[1].text, "*- ");
        assert_eq!(actual[1].spans[0].fg, Some(NUMERIC_ANNOTATION));
        assert_eq!(actual[1].spans[1].fg, Some(NUMERIC_ANNOTATION));
        assert_eq!(actual[1].spans[2].fg, Some(NUMERIC_ANNOTATION));
        assert_eq!(actual[1].spans[3].text, "^");
        assert_eq!(actual[1].spans[3].fg, Some(NUMERIC_ANNOTATION_CARET));
        assert_eq!(actual[1].spans[4].text, "-2");
        assert_eq!(actual[1].spans[4].fg, Some("magenta"));
        assert_eq!(actual[1].spans[6].text, "^");
        assert_eq!(actual[1].spans[6].fg, Some(NUMERIC_ANNOTATION_CARET));
        assert_eq!(actual[1].spans[7].text, "+8");
        assert_eq!(actual[1].spans[7].fg, Some("cyan"));
        assert_eq!(actual[3].spans[0].fg, Some(NUMERIC_ANNOTATION));
        assert_eq!(actual[3].spans[1].fg, Some(NUMERIC_ANNOTATION));
    }

    #[test]
    fn keeps_equal_lines_out_when_only_diffline_is_enabled() {
        let actual = generate_numeric_diff(
            "same\nretry=5 timeout=92\n",
            "same\nretry=3 timeout=100\n",
            true,
            true,
            false,
        );

        assert_eq!(actual.len(), 4);
        assert_eq!(actual[0].spans[0].text, "2 | ");
        assert_eq!(actual[3].spans[1].text, "*+ ");
    }

    #[test]
    fn ignores_whitespace_alignment_differences_around_numbers() {
        let actual = describe_numeric_delta(
            "-rw-r--r--  1 blacknon  staff   999 Apr 17 18:37 file.txt",
            "-rw-r--r--  1 blacknon  staff  1000 Apr 17 18:37 file.txt",
        );

        assert_eq!(
            actual,
            Some(vec![NumericDelta {
                start: 31,
                before_delta: "-1".to_string(),
                after_delta: "+1".to_string(),
            }])
        );
    }

    #[test]
    fn strips_ansi_from_changed_lines_even_when_color_is_enabled() {
        let actual = generate_numeric_diff(
            "\u{1b}[32mvalue=2\u{1b}[0m\n",
            "\u{1b}[31mvalue=1\u{1b}[0m\n",
            true,
            false,
            true,
        );

        assert_eq!(actual[0].spans[2].text, "value=1");
        assert_eq!(actual[2].spans[2].text, "value=2");
    }

    #[test]
    fn preserves_ansi_on_equal_lines_only_when_color_is_enabled() {
        let actual = generate_numeric_diff(
            "\u{1b}[32msame\u{1b}[0m\n",
            "\u{1b}[32msame\u{1b}[0m\n",
            false,
            false,
            true,
        );
        assert_eq!(actual[0].spans[0].text, "\u{1b}[32msame\u{1b}[0m");

        let stripped = generate_numeric_diff(
            "\u{1b}[32msame\u{1b}[0m\n",
            "\u{1b}[32msame\u{1b}[0m\n",
            false,
            false,
            false,
        );
        assert_eq!(stripped[0].spans[0].text, "same");
    }
}
