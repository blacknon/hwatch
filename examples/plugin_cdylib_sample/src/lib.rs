use std::ffi::c_char;
use std::slice;
use std::str;

const ABI_VERSION: u32 = 1;
const OUTPUT_BATCH: u32 = 0;
const OUTPUT_WATCH: u32 = 1;

static PLUGIN_NAME: &[u8] = b"summary-diff\0";
static HEADER_TEXT: &[u8] = b"Summary\0";

#[repr(C)]
pub struct HwatchSlice {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
pub struct HwatchOwnedBytes {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct HwatchDiffRequest {
    pub dest: HwatchSlice,
    pub src: HwatchSlice,
    pub output_kind: u32,
    pub color: bool,
    pub line_number: bool,
    pub only_diffline: bool,
}

#[repr(C)]
pub struct HwatchPluginMetadata {
    pub abi_version: u32,
    pub supports_only_diffline: bool,
    pub plugin_name: *const c_char,
    pub header_text: *const c_char,
}

#[no_mangle]
pub extern "C" fn hwatch_diffmode_metadata() -> HwatchPluginMetadata {
    HwatchPluginMetadata {
        abi_version: ABI_VERSION,
        supports_only_diffline: true,
        plugin_name: PLUGIN_NAME.as_ptr().cast(),
        header_text: HEADER_TEXT.as_ptr().cast(),
    }
}

#[no_mangle]
pub extern "C" fn hwatch_diffmode_generate(req: HwatchDiffRequest) -> HwatchOwnedBytes {
    let dest = unsafe { slice_to_str(req.dest) }.unwrap_or_default();
    let src = unsafe { slice_to_str(req.src) }.unwrap_or_default();

    let lines = generate_summary_lines(
        dest,
        src,
        req.output_kind,
        req.line_number,
        req.only_diffline,
    );

    let json = render_json_response("Summary", &lines);
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

fn generate_summary_lines(
    dest: &str,
    src: &str,
    output_kind: u32,
    line_number: bool,
    only_diffline: bool,
) -> Vec<String> {
    let dest_lines: Vec<&str> = dest.lines().collect();
    let src_lines: Vec<&str> = src.lines().collect();
    let max_len = dest_lines.len().max(src_lines.len());

    let mut results = Vec::new();

    for index in 0..max_len {
        let before = src_lines.get(index).copied().unwrap_or("");
        let after = dest_lines.get(index).copied().unwrap_or("");

        if before == after && only_diffline {
            continue;
        }

        let prefix = if line_number {
            format!("{:>3} | ", index + 1)
        } else {
            String::new()
        };

        let body = match (before.is_empty(), after.is_empty(), before == after) {
            (_, _, true) => format!("  {after}"),
            (true, false, false) => format!("+ {after}"),
            (false, true, false) => format!("- {before}"),
            (false, false, false) => {
                let delta = after.chars().count() as isize - before.chars().count() as isize;
                match output_kind {
                    OUTPUT_BATCH => format!("~ {before} -> {after} (delta {delta:+})"),
                    OUTPUT_WATCH => format!("~ {before} => {after} (delta {delta:+})"),
                    _ => format!("~ {before} -> {after}"),
                }
            }
            (true, true, false) => String::new(),
        };

        results.push(format!("{prefix}{body}"));
    }

    results
}

fn render_json_response(header_text: &str, lines: &[String]) -> String {
    let mut json = String::from("{\"schema_version\":1,\"header_text\":\"");
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
