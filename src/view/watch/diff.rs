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
                    COLOR_ELEMENT_D,
                    COLOR_ELEMENT_D,
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

pub struct LineDiff {
    pub line: i32,
    dataset: Vec<Color>,
    color: bool,
}

impl LineDiff {
    pub fn new(color: bool) -> Self {
        Self {
            line: 0,
            dataset: vec![],
            color: color,
        }
    }

    pub fn create_dataset(&mut self, data1: String, data2: String) {
        // Compare both before/after output.
        let Changeset { diffs, .. } = Changeset::new(&data1.clone(), &data2.clone(), "\n");
        let mut dataset: Vec<Color> = Vec::new();

        // for diffs
        for i in 0..diffs.len() {
            match diffs[i] {
                Difference::Same(ref diff_data) => {
                    for line in diff_data.lines() {
                        // push line header
                        let mut header = Color {
                            ansi: (0, 1, 1),
                            data: "   ".to_string(),
                        };
                        dataset.push(header);

                        dataset.append(&mut self.craete_dataset_element(&format!("{}\n", line), 0));
                        self.line += 1;
                    }
                }
                Difference::Add(ref diff_data) => {
                    for line in diff_data.lines() {
                        // push line header
                        let mut header = Color {
                            ansi: (0, COLOR_ELEMENT_G.into(), 1),
                            data: "+  ".to_string(),
                        };
                        dataset.push(header);

                        dataset.append(&mut self.craete_dataset_element(&format!("{}\n", line), 1));
                        self.line += 1;
                    }
                }
                Difference::Rem(ref diff_data) => {
                    for line in diff_data.lines() {
                        // push line header
                        let mut header = Color {
                            ansi: (0, COLOR_ELEMENT_R.into(), 1),
                            data: "-  ".to_string(),
                        };
                        dataset.push(header);

                        dataset
                            .append(&mut self.craete_dataset_element(&format!("{}\n", line), -1));
                        self.line += 1;
                    }
                }
            }
        }
        self.dataset = dataset;
    }

    fn craete_dataset_element(&mut self, line: &str, status: i32) -> Vec<Color> {
        let mut result: Vec<Color> = vec![];

        if self.color {
            let pairs = ansi_parse(line);

            for mut pair in pairs {
                if pair.ansi == (0, 1, 1) {
                    match status {
                        1 => pair.ansi = (0, COLOR_ELEMENT_G.into(), 1),
                        -1 => pair.ansi = (0, COLOR_ELEMENT_R.into(), 1),
                        _ => {}
                    }
                }

                result.push(pair);
            }
        } else {
            let mut ansi = (0, 1, 1);
            match status {
                0 => ansi = (0, 1, 1),
                1 => ansi = (0, COLOR_ELEMENT_G.into(), 1),
                -1 => ansi = (0, COLOR_ELEMENT_R.into(), 1),
                _ => {}
            }

            let data = Color {
                ansi: ansi,
                data: line.to_string(),
            };
            result.push(data);
        }

        return result;
    }

    pub fn print(&mut self, mut watch: WatchPad) {
        for data in &self.dataset {
            let flag = data.ansi.0;
            let fg_color = data.ansi.1 as i16;
            let bg_color = data.ansi.2 as i16;

            watch.print(data.data.clone(), fg_color, bg_color, vec![flag]);
        }
    }
}

// fn line_diff_print() {}

// pub fn word_diff(mut watch: WatchPad, before_output: String, after_output: String) {
//
// }
