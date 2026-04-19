use std::slice;
use std::str;

use hwatch_diffmode::{
    PluginDiffRequest as HwatchDiffRequest, PluginMetadata as HwatchPluginMetadata,
    PluginOwnedBytes as HwatchOwnedBytes, PluginSlice as HwatchSlice, PLUGIN_ABI_VERSION,
};
use similar::{ChangeTag, TextDiff};

static PLUGIN_NAME: &[u8] = b"line-num-diff\0";
static HEADER_TEXT: &[u8] = b"LineNum\0";

#[derive(Clone)]
struct NumericToken {
    raw: String,
    value: f64,
}

#[derive(Clone)]
struct RenderedLine {
    line_no: Option<usize>,
    marker: &'static str,
    text: String,
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

    let lines = generate_line_num_diff(dest, src, req.line_number, req.only_diffline);
    let header_text = if req.only_diffline {
        "LineNum(Only)"
    } else {
        "LineNum"
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

fn generate_line_num_diff(
    dest: &str,
    src: &str,
    line_number: bool,
    only_diffline: bool,
) -> Vec<String> {
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

        if change.tag() == ChangeTag::Delete
            && index + 1 < changes.len()
            && changes[index + 1].tag() == ChangeTag::Insert
        {
            let before = trim_line_end(change.to_string());
            let after = trim_line_end(changes[index + 1].to_string());

            if let Some(delta_lines) = describe_numeric_delta(&before, &after) {
                for delta_line in delta_lines {
                    rendered.push(RenderedLine {
                        line_no: None,
                        marker: "^  ",
                        text: delta_line,
                    });
                }
            }

            rendered.push(RenderedLine {
                line_no: change.old_index().map(|value| value + 1),
                marker: "-  ",
                text: before,
            });
            rendered.push(RenderedLine {
                line_no: changes[index + 1].new_index().map(|value| value + 1),
                marker: "+  ",
                text: after,
            });
            index += 2;
            continue;
        }

        match change.tag() {
            ChangeTag::Equal => {
                if !only_diffline {
                    rendered.push(RenderedLine {
                        line_no: change.old_index().map(|value| value + 1),
                        marker: "   ",
                        text: trim_line_end(change.to_string()),
                    });
                }
            }
            ChangeTag::Delete => rendered.push(RenderedLine {
                line_no: change.old_index().map(|value| value + 1),
                marker: "-  ",
                text: trim_line_end(change.to_string()),
            }),
            ChangeTag::Insert => rendered.push(RenderedLine {
                line_no: change.new_index().map(|value| value + 1),
                marker: "+  ",
                text: trim_line_end(change.to_string()),
            }),
        }

        index += 1;
    }

    rendered
        .into_iter()
        .map(|line| render_line(line, line_number, line_width))
        .collect()
}

fn render_line(line: RenderedLine, line_number: bool, line_width: usize) -> String {
    let prefix = if line_number {
        match line.line_no {
            Some(number) => format!("{number:>line_width$} | "),
            None => format!("{:>line_width$} | ", ""),
        }
    } else {
        String::new()
    };

    format!("{prefix}{}{}", line.marker, line.text)
}

fn trim_line_end(mut line: String) -> String {
    if line.ends_with('\n') {
        line.pop();
    }
    line
}

fn describe_numeric_delta(before: &str, after: &str) -> Option<Vec<String>> {
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

        let delta = after_token.value - before_token.value;
        deltas.push(format!(
            "{} -> {} ({})",
            before_token.raw,
            after_token.raw,
            format_signed_number(delta)
        ));
    }

    if deltas.is_empty() {
        None
    } else {
        Some(
            deltas
                .into_iter()
                .map(|delta| format!("numeric delta: {delta}"))
                .collect(),
        )
    }
}

fn extract_numeric_tokens(input: &str) -> (Vec<NumericToken>, String) {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    let mut skeleton = String::new();
    let mut tokens = Vec::new();

    while index < chars.len() {
        if let Some((end, raw)) = parse_number_token(&chars, index) {
            let value = raw.parse::<f64>().ok();
            if let Some(value) = value {
                tokens.push(NumericToken {
                    raw: raw.clone(),
                    value,
                });
                skeleton.push('#');
                index = end;
                continue;
            }
        }

        skeleton.push(chars[index]);
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

fn render_json_response(header_text: &str, lines: &[String]) -> String {
    let mut json = format!(
        "{{\"schema_version\":{},\"header_text\":\"",
        PLUGIN_ABI_VERSION
    );
    json.push_str(&escape_json(header_text));
    json.push_str("\",\"lines\":[");

    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&escape_json(line));
        json.push('"');
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
    fn inserts_numeric_delta_line_for_replaced_line() {
        let actual = generate_line_num_diff("count: 15\n", "count: 10\n", true, false);

        assert_eq!(
            actual,
            vec![
                "  | ^  numeric delta: 10 -> 15 (+5)",
                "1 | -  count: 10",
                "1 | +  count: 15",
            ]
        );
    }

    #[test]
    fn keeps_equal_lines_out_when_only_diffline_is_enabled() {
        let actual = generate_line_num_diff("same\ncount: 15\n", "same\ncount: 10\n", true, true);

        assert_eq!(
            actual,
            vec![
                "  | ^  numeric delta: 10 -> 15 (+5)",
                "2 | -  count: 10",
                "2 | +  count: 15",
            ]
        );
    }
}
