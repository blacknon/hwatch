extern crate difference;

use self::difference::{Changeset, Difference};
use std::cmp;
use view::watch::watch::WatchPad;

// watch type diff
pub fn watch_diff(mut watch: WatchPad, data1: String, data2: String) {
    // output to vector
    let mut data1_lines: Vec<&str> = data1.lines().collect();
    let mut data2_lines: Vec<&str> = data2.lines().collect();

    // get max line
    let max_line = cmp::max(data1_lines.len(), data2_lines.len());

    for i in 0..max_line {
        // push empty line
        if data1_lines.len() <= i {
            data1_lines.push("");
        }
        if data2_lines.len() <= i {
            data2_lines.push("");
        }

        if data1_lines[i] != data2_lines[i] {
            let mut data1_chars: Vec<char> = data1_lines[i].chars().collect();
            let mut data2_chars: Vec<char> = data2_lines[i].chars().collect();

            let max_char = cmp::max(data1_chars.len(), data2_chars.len());

            for x in 0..max_char {
                let space: char = ' ';

                if data1_chars.len() <= max_char {
                    data1_chars.push(space);
                }
                if data2_chars.len() <= max_char {
                    data2_chars.push(space);
                }

                if data1_chars[x] != data2_chars[x] {
                    watch.print_watch_data(data2_chars[x].to_string(), true, 0);
                } else {
                    watch.print_watch_data(data2_chars[x].to_string(), false, 0);
                }
            }
            watch.print_watch_data("\n".to_string(), false, 0);
        } else {
            watch.print_watch_data(data2_lines[i].to_string(), false, 0);
            watch.print_watch_data("\n".to_string(), false, 0);
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
                    watch.print_watch_data(format!("  {}\n", line), false, 0);
                }
            }
            Difference::Add(ref diff_data) => {
                for line in diff_data.lines() {
                    watch.print_watch_data(format!("+ {}\n", line), false, 2);
                }
            }
            Difference::Rem(ref diff_data) => {
                for line in diff_data.lines() {
                    watch.print_watch_data(format!("- {}\n", line), false, 3);
                }
            }
        }
    }
}

// pub fn word_diff(mut watch: WatchPad, before_output: String, after_output: String) {
//
// }
