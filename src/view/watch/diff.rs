extern crate difference;

use self::difference::{Changeset, Difference};
use std::cmp;

use view::color::*;
use view::watch::watch::WatchPad;

pub struct Diff {
    pub color: bool,
}

impl Diff {
    pub fn set(color: bool) -> Self {
        Self { color: color }
    }

    pub fn watch_diff(&mut self, mut watch: WatchPad, data1: String, data2: String) {
        // output to vector
        let mut lines1: Vec<&str> = data1.lines().collect();
        let mut lines2: Vec<&str> = data2.lines().collect();

        // get max line
        let max_line = cmp::max(lines1.len(), lines2.len());

        // for max_line
        for i in 0..max_line {
            // push empty line
            if lines1.len() <= i {
                lines1.push("");
            }
            if lines2.len() <= i {
                lines2.push("");
            }

            if self.color {
                // ANSIコードのdiffは取得するのは辛いので、出力文字列だけをターゲットとする。
                // つまり、一度カラーセット単位にして、line2をベースにしたdiffを取るようにする
                // self.watch_diff_color_print_line(ansi,lines1[i],lines2[i]);
                let lines1_colorset = ansi_parse(lines1[i]);
                let lines2_colorset = ansi_parse(lines2[i]);
            } else {
                // print line data
                self.watch_diff_print_line(
                    COLOR_ELEMENT_D,
                    COLOR_ELEMENT_D,
                    watch.clone(),
                    lines1[i],
                    lines2[i],
                );

                // print newline
                watch.print("\n".to_string(), COLOR_ELEMENT_D, COLOR_ELEMENT_D, vec![]);
            }
        }
    }

    fn watch_diff_print_line(
        &mut self,
        fg: i16,
        bg: i16,
        mut watch: WatchPad,
        line1: &str,
        line2: &str,
    ) {
        if line1 != line2 {
            // diff line
            let mut chars1: Vec<char> = line1.chars().collect();
            let mut chars2: Vec<char> = line2.chars().collect();

            let max_char = cmp::max(chars1.len(), chars2.len());

            for x in 0..max_char {
                let space: char = ' ';

                if chars1.len() <= max_char {
                    chars1.push(space);
                }
                if chars2.len() <= max_char {
                    chars2.push(space);
                }

                if chars1[x] != chars2[x] {
                    // if diff => print default color
                    watch.print(
                        chars2[x].to_string(),
                        COLOR_ELEMENT_D,
                        COLOR_ELEMENT_D,
                        vec![IS_REVERSE],
                    );
                } else {
                    watch.print(chars2[x].to_string(), fg, bg, vec![]);
                }
            }
        } else {
            // same line
            watch.print(line2.to_string(), fg, bg, vec![]);
        }
    }

    fn watch_diff_color_print_line(&mut self, line1: &str, line2: &str) {}
}

// color出力をどうしてやるときれいになるだろうか…？？
// とりあえず、行単位でprintはするようにして、前の行のcolorをforで扱うことで前の行からの色の続きを取得させてやれば対応はできそうだ。

// watch type diff
// @TODO: Color対応を追加
//     colorフラグを引数に追加し、もし有効だった場合は出力時にパースして処理するように定義する
//
//     struct化が必要な気がする。
//
// @Note:
//     通常のwatchコマンドでは、ansiコードが変わっても特に表示の変化はなかった。
//     つまり、こちらのwatch_diffも同様に処理してやればいいと思われる。
pub fn watch_diff(mut watch: WatchPad, data1: String, data2: String, _color: bool) {
    let fg_color = COLOR_ELEMENT_D;
    let bg_color = COLOR_ELEMENT_D;

    // output to vector
    let mut data1_lines: Vec<&str> = data1.lines().collect();
    let mut data2_lines: Vec<&str> = data2.lines().collect();

    // get max line
    let max_line = cmp::max(data1_lines.len(), data2_lines.len());
    // @TODO: forで各行の処理をする際にcolor parseをして、その結果の配列の最後の値からその行の最後のansiを取得する。
    //        そのansiを利用して次の行のカラーを指定することで複数行にまたがるANSI Colorを再現させる。

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
                    watch.print(
                        data2_chars[x].to_string(),
                        fg_color,
                        bg_color,
                        vec![IS_REVERSE],
                    );
                } else {
                    watch.print(data2_chars[x].to_string(), fg_color, bg_color, vec![]);
                }
            }
            watch.print("\n".to_string(), fg_color, bg_color, vec![]);
        } else {
            watch.print(data2_lines[i].to_string(), fg_color, bg_color, vec![]);
            watch.print("\n".to_string(), fg_color, bg_color, vec![]);
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
// @TODO: Color対応を追加
//     colorフラグを引数に追加し、もし有効だった場合は出力時にパースして処理するように定義する
pub fn line_diff(mut watch: WatchPad, before_output: String, after_output: String, _color: bool) {
    let Changeset { diffs, .. } =
        Changeset::new(&before_output.clone(), &after_output.clone(), "\n");

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref diff_data) => {
                for line in diff_data.lines() {
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

// pub fn word_diff(mut watch: WatchPad, before_output: String, after_output: String) {
//
// }
