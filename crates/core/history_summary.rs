// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use rayon::prelude::*;
use similar::{Change, ChangeTag, TextDiff};
use std::sync::{Arc, Mutex};

use super::HistorySummary;

impl HistorySummary {
    pub fn calc(&mut self, src: &str, dest: &str, enable_char_diff: bool, ignore_spaceblock: bool) {
        // reset
        self.line_add = 0;
        self.line_rem = 0;
        self.char_add = 0;
        self.char_rem = 0;

        let line_add = Arc::new(Mutex::new(0));
        let line_rem = Arc::new(Mutex::new(0));
        let char_add = Arc::new(Mutex::new(0));
        let char_rem = Arc::new(Mutex::new(0));

        let src = if ignore_spaceblock {
            hwatch_diffmode::normalize_space_blocks(src)
        } else {
            src.to_string()
        };
        let dest = if ignore_spaceblock {
            hwatch_diffmode::normalize_space_blocks(dest)
        } else {
            dest.to_string()
        };

        let line_diff = TextDiff::from_lines(&src, &dest);

        line_diff.ops().par_iter().for_each(|l_op| {
            for change in line_diff.iter_inline_changes(l_op) {
                let mut line_add_lock = line_add.lock().unwrap();
                let mut line_rem_lock = line_rem.lock().unwrap();
                match change.tag() {
                    ChangeTag::Insert => {
                        *line_add_lock += 1;
                    }
                    ChangeTag::Delete => {
                        *line_rem_lock += 1;
                    }
                    ChangeTag::Equal => {}
                }
            }
        });

        if enable_char_diff {
            let mut char_add_lock = char_add.lock().unwrap();
            let mut char_rem_lock = char_rem.lock().unwrap();
            let (char_add, char_rem) = self::calc_char_diff(line_diff.iter_all_changes().collect());

            *char_add_lock = char_add as u64;
            *char_rem_lock = char_rem as u64;
        }

        self.line_add = *line_add.lock().unwrap();
        self.line_rem = *line_rem.lock().unwrap();
        self.char_add = *char_add.lock().unwrap();
        self.char_rem = *char_rem.lock().unwrap();
    }
}

pub(super) fn calc_char_diff(change_set: Vec<Change<&str>>) -> (usize, usize) {
    let mut char_add = 0;
    let mut char_rem = 0;

    let mut remove_changes = Vec::new();
    let mut insert_changes = Vec::new();
    let mut previous_tag = ChangeTag::Equal;

    for change in change_set {
        match change.tag() {
            ChangeTag::Delete => {
                if previous_tag == ChangeTag::Insert && !remove_changes.is_empty() {
                    get_char_diff(
                        &mut remove_changes,
                        &mut insert_changes,
                        &mut char_add,
                        &mut char_rem,
                    );
                }
                remove_changes.push(change);
            }
            ChangeTag::Insert => {
                if previous_tag == ChangeTag::Delete && !insert_changes.is_empty() {
                    get_char_diff(
                        &mut remove_changes,
                        &mut insert_changes,
                        &mut char_add,
                        &mut char_rem,
                    );
                }
                insert_changes.push(change);
            }
            ChangeTag::Equal => {
                if previous_tag != ChangeTag::Equal {
                    get_char_diff(
                        &mut remove_changes,
                        &mut insert_changes,
                        &mut char_add,
                        &mut char_rem,
                    );
                }
            }
        }
        previous_tag = change.tag();
    }

    if !remove_changes.is_empty() || !insert_changes.is_empty() {
        get_char_diff(
            &mut remove_changes,
            &mut insert_changes,
            &mut char_add,
            &mut char_rem,
        );
    }

    (char_add, char_rem)
}

fn get_char_diff(
    remove_changes: &mut Vec<Change<&str>>,
    insert_changes: &mut Vec<Change<&str>>,
    char_add: &mut usize,
    char_rem: &mut usize,
) {
    let remove_string: String = remove_changes.iter().map(|c| c.value()).collect();
    let insert_string: String = insert_changes.iter().map(|c| c.value()).collect();

    let char_diff_set = TextDiff::from_chars(&remove_string, &insert_string);
    for char_change in char_diff_set.iter_all_changes() {
        let length = char_change.value().len();
        match char_change.tag() {
            ChangeTag::Insert => *char_add += length,
            ChangeTag::Delete => *char_rem += length,
            _ => {}
        }
    }

    remove_changes.clear();
    insert_changes.clear();
}
