extern crate difference;

use self::difference::{Changeset, Difference};
use std::cmp;

use view::color::*;
use view::watch::watch::WatchPad;

pub fn watch_diff(mut watch: WatchPad, data1: String, data2: String, color: bool) {
    // output to vector
    let mut data1_vec: Vec<&str> = data1.lines().collect();
    let mut data2_vec: Vec<&str> = data2.lines().collect();

    // get max line
    let max_line = cmp::max(data1_vec.len(), data2_vec.len());

    // for diff lines
    for i in 0..max_line {
        // push empty line
        if data1_vec.len() <= i {
            data1_vec.push("");
        }
        if data2_vec.len() <= i {
            data2_vec.push("");
        }

        if color {
            // watch: color print
            // get color set
            let data1_pair = ansi_parse(data1_vec[i]);
            let data2_pair = ansi_parse(data2_vec[i]);

            // get max pair count
            let max_pair = cmp::max(data1_pair.len(), data2_pair.len());

            for c in 0..max_pair {
                // let mut data1_pair_ansi = (0, 1, 1);
                let mut data1_pair_str = "";
                if data1_pair.len() > c {
                    // data1_pair_ansi = data1_pair[c].ansi;
                    data1_pair_str = &data1_pair[c].data;
                }

                let mut data2_pair_ansi = (0, 1, 1);
                let mut data2_pair_str = "";
                if data2_pair.len() > c {
                    data2_pair_ansi = data2_pair[c].ansi;
                    data2_pair_str = &data2_pair[c].data;
                }

                // print watch
                watch_diff_print(
                    watch.clone(),
                    data2_pair_ansi,
                    data1_pair_str,
                    data2_pair_str,
                );
            }
        } else {
            // watch: plane print
            watch_diff_print(
                watch.clone(),
                (0, COLOR_ELEMENT_D.into(), COLOR_ELEMENT_D.into()),
                data1_vec[i],
                data2_vec[i],
            );
        }
        watch.print("\n".to_string(), COLOR_ELEMENT_D, COLOR_ELEMENT_D, vec![]);
    }
}

fn watch_diff_print(mut watch: WatchPad, ansi: (i32, i32, i32), data1: &str, data2: &str) {
    let flag = ansi.0;
    let fg_color = ansi.1 as i16;
    let bg_color = ansi.2 as i16;
    if data1 != data2 {
        let mut data1_chars: Vec<char> = data1.chars().collect();
        let mut data2_chars: Vec<char> = data2.chars().collect();

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
                watch.print(
                    data2_chars[x].to_string(),
                    fg_color,
                    bg_color,
                    vec![IS_REVERSE],
                );
            } else {
                watch.print(data2_chars[x].to_string(), fg_color, bg_color, vec![flag]);
            }
        }
    } else {
        watch.print(data2.to_string(), fg_color, bg_color, vec![flag]);
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
// @TODO: Color対応を追加
//     colorフラグを引数に追加し、もし有効だった場合は出力時にパースして処理するように定義する
pub fn line_diff(mut watch: WatchPad, before_output: String, after_output: String, _color: bool) {
    let Changeset { diffs, .. } =
        Changeset::new(&before_output.clone(), &after_output.clone(), "\n");

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
                    // color対応は、この単位で処理
                    watch.print(
                        format!("  {}\n", line),
                        COLOR_ELEMENT_D,
                        COLOR_ELEMENT_D,
                        vec![],
                    );
                }
            }
            Difference::Add(ref diff_data) => {
                for line in diff_data.lines() {
                    // color対応は、この単位で処理
                    watch.print(
                        format!("+ {}\n", line),
                        COLOR_ELEMENT_G,
                        COLOR_ELEMENT_D,
                        vec![],
                    );
                }
            }
            Difference::Rem(ref diff_data) => {
                for line in diff_data.lines() {
                    // color対応は、この単位で処理
                    watch.print(
                        format!("- {}\n", line),
                        COLOR_ELEMENT_R,
                        COLOR_ELEMENT_D,
                        vec![],
                    );
                }
            }
        }
    }
}

fn line_diff_print() {}

// pub fn word_diff(mut watch: WatchPad, before_output: String, after_output: String) {
//
// }
