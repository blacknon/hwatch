// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use hwatch_diffmode::DiffMode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use unicode_width::UnicodeWidthStr;

pub fn register_diff_mode_name(
    diff_mode_name_to_index: &mut HashMap<String, usize>,
    name: String,
    index: usize,
) -> Result<(), String> {
    if diff_mode_name_to_index.contains_key(&name) {
        return Err(format!("duplicate diff mode name: '{name}'"));
    }

    diff_mode_name_to_index.insert(name, index);
    Ok(())
}

pub fn calculate_diff_mode_header_width(diff_modes: &[Arc<Mutex<Box<dyn DiffMode>>>]) -> usize {
    let mut max_width = 0;

    for diff_mode in diff_modes {
        let mut diff_mode = diff_mode.lock().unwrap();
        for only_diffline in [false, true] {
            let mut options = hwatch_diffmode::DiffModeOptions::new();
            options.set_only_diffline(only_diffline);
            diff_mode.set_option(options);
            let header_text = diff_mode.get_header_text();
            max_width = max_width.max(UnicodeWidthStr::width(header_text.as_str()));
        }
    }

    max_width
}

#[cfg(test)]
mod tests {
    use super::register_diff_mode_name;
    use std::collections::HashMap;

    #[test]
    fn register_diff_mode_name_rejects_duplicates() {
        let mut diff_mode_name_to_index = HashMap::new();
        register_diff_mode_name(&mut diff_mode_name_to_index, "watch".to_string(), 1).unwrap();

        let error = register_diff_mode_name(&mut diff_mode_name_to_index, "watch".to_string(), 2)
            .unwrap_err();

        assert_eq!(error, "duplicate diff mode name: 'watch'");
    }
}
