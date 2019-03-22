extern crate difference;

use self::difference::{Changeset, Difference};
use std::cmp;
use view::watch::window::WatchPad;

// watch type diff
pub fn watch_diff(mut watch: WatchPad, before_output: String, after_output: String) {
    // before and after output to vector
    let mut before_lines: Vec<&str> = before_output.lines().collect();
    let mut after_lines: Vec<&str> = after_output.lines().collect();

    // get max line before and after output
    let max_line = cmp::max(before_lines.len(), after_lines.len());

    for i in 0..max_line {
        if before_lines.len() <= i {
            before_lines.push("");
        }
        if after_lines.len() <= i {
            after_lines.push("");
        }

        if before_lines[i] != after_lines[i] {
            let mut before_chars: Vec<char> = before_lines[i].chars().collect();
            let mut after_chars: Vec<char> = after_lines[i].chars().collect();

            let max_char = cmp::max(before_chars.len(), after_chars.len());

            for x in 0..max_char {
                let space: char = ' ';

                if before_chars.len() <= max_char {
                    before_chars.push(space);
                }
                if after_chars.len() <= max_char {
                    after_chars.push(space);
                }

                if before_chars[x] != after_chars[x] {
                    watch.update_output_pad_char(after_chars[x].to_string(), true, 0);
                } else {
                    watch.update_output_pad_char(after_chars[x].to_string(), false, 0);
                }
            }
            watch.update_output_pad_char("\n".to_string(), false, 0);
        } else {
            watch.update_output_pad_char(after_lines[i].to_string(), false, 0);
            watch.update_output_pad_char("\n".to_string(), false, 0);
        }
    }
}

// line type diff get strings
pub fn line_diff_str_get(before_output: String, after_output: String) -> String {
    // Compare both before/after output.
    let Changeset { diffs, .. } =
        Changeset::new(&before_output.clone(), &after_output.clone(), "\n");

    // Create result output (strings)
    let mut result_vec: Vec<String> = Vec::new();
    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
                    result_vec.push(format!("  {}", line));
                }
            }
            Difference::Add(ref diff_data) => {
                for line in diff_data.lines() {
                    result_vec.push(format!("+  {}", line));
                }
            }
            Difference::Rem(ref diff_data) => {
                for line in diff_data.lines() {
                    result_vec.push(format!("-  {}", line));
                }
            }
        }
    }
    let result_string = result_vec.join("\n");
    return result_string;
}

// line type diff
pub fn line_diff(mut watch: WatchPad, before_output: String, after_output: String) {
    let Changeset { diffs, .. } =
        Changeset::new(&before_output.clone(), &after_output.clone(), "\n");

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
                    watch.update_output_pad_char(format!("  {}\n", line), false, 0);
                }
            }
            Difference::Add(ref diff_data) => {
                for line in diff_data.lines() {
                    watch.update_output_pad_char(format!("+ {}\n", line), false, 2);
                }
            }
            Difference::Rem(ref diff_data) => {
                for line in diff_data.lines() {
                    watch.update_output_pad_char(format!("- {}\n", line), false, 3);
                }
            }
        }
    }
}

// pub fn word_diff(mut watch: WatchPad, before_output: String, after_output: String) {
//
// }
