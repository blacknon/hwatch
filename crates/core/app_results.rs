// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use super::{App, ResultItems};
use crate::common::{logging_result, OutputMode};
use crate::exec::{exec_after_command, CommandResult};
use crate::history::{History, HistorySummary};
use crate::output::WatchRenderData;
use hwatch_diffmode::text_eq_ignoring_space_blocks;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::thread;

impl App<'_> {
    pub(super) fn set_output_data(&mut self, num: usize) {
        let results = match self.output_mode {
            OutputMode::Output => &self.results,
            OutputMode::Stdout => &self.results_stdout,
            OutputMode::Stderr => &self.results_stderr,
        };

        if results.is_empty() {
            return;
        }

        let mut target_dst: usize = num;

        if target_dst == 0 {
            target_dst = get_results_latest_index(results);
        } else {
            target_dst = get_near_index(results, target_dst);
        }
        let previous_dst = get_results_previous_index(results, target_dst);

        let dest: &CommandResult = &results[&target_dst].command_result;
        let mut src = dest;

        let support_only_diffline = self.diff_modes[self.diff_mode]
            .lock()
            .unwrap()
            .get_support_only_diffline();

        if previous_dst > 0 {
            src = &results[&previous_dst].command_result;
        } else if previous_dst == 0 && self.is_only_diffline && support_only_diffline {
            src = &results[&0].command_result;
        }

        let output_data = self.printer.get_watch_data(dest, src);
        self.apply_watch_render_data(output_data);
    }

    pub(super) fn apply_watch_render_data(&mut self, render_data: WatchRenderData) {
        match render_data {
            WatchRenderData::SinglePane(pane) => {
                self.watch_area.is_line_number = pane.is_line_number;
                self.watch_area.is_line_diff_head = pane.is_line_diff_head;
                self.watch_area.update_output(pane.lines);
            }
        }
    }

    pub(super) fn refresh_selected_watch_output(&mut self) {
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub(super) fn delete_output_data(&mut self, num: usize) {
        let selected = self.history_area.get_state_select();
        if num != 0 && self.history_area.get_history_size() > 0 {
            let results = match self.output_mode {
                OutputMode::Output => &mut self.results,
                OutputMode::Stdout => &mut self.results_stdout,
                OutputMode::Stderr => &mut self.results_stderr,
            };

            results.remove(&num);
            self.history_area.delete(num);

            let new_selected = self.reset_history(selected);

            self.set_output_data(new_selected);
        }
    }

    pub(super) fn clear_history_except_selected(&mut self) {
        let selected = self.history_area.get_state_select();
        retain_selected_and_latest_result_only(&mut self.results, selected);
        retain_selected_and_latest_result_only(&mut self.results_stdout, selected);
        retain_selected_and_latest_result_only(&mut self.results_stderr, selected);

        let new_selected = self.reset_history(selected);
        self.set_output_data(new_selected);
    }

    pub(super) fn reset_history(&mut self, selected: usize) -> usize {
        let results = match self.output_mode {
            OutputMode::Output => &self.results,
            OutputMode::Stdout => &self.results_stdout,
            OutputMode::Stderr => &self.results_stderr,
        };

        let mut tmp_history = vec![];
        let latest_num: usize = get_results_latest_index(results);
        tmp_history.push(History {
            timestamp: "latest                 ".to_string(),
            status: results[&latest_num].command_result.status,
            num: 0,
            summary: HistorySummary::init(),
        });

        let mut new_select: Option<usize> = None;
        let mut results_vec = results.iter().collect::<Vec<(&usize, &ResultItems)>>();
        results_vec.sort_by_key(|&(key, _)| key);

        let mut tmp_results: HashMap<usize, ResultItems> = HashMap::new();
        for (key, result) in results_vec {
            if key == &0 {
                continue;
            }

            let support_only_diffline: bool = self.diff_modes[self.diff_mode]
                .lock()
                .unwrap()
                .get_support_only_diffline();

            let mut is_push = true;
            if self.is_filtered {
                let result_text = match (
                    self.output_mode,
                    support_only_diffline,
                    self.is_only_diffline,
                ) {
                    (OutputMode::Output | OutputMode::Stdout | OutputMode::Stderr, true, true) => {
                        result.get_diff_only_data(self.ansi_color)
                    }
                    (OutputMode::Output, _, _) => result.command_result.get_output(),
                    (OutputMode::Stdout, _, _) => result.command_result.get_stdout(),
                    (OutputMode::Stderr, _, _) => result.command_result.get_stderr(),
                };

                is_push = self.matches_filter_text(&result_text);
            }

            if is_push {
                tmp_history.push(History {
                    timestamp: result.command_result.timestamp.clone(),
                    status: result.command_result.status,
                    num: *key as u16,
                    summary: result.summary.clone(),
                });

                tmp_results.insert(*key, result.clone());

                if &selected == key {
                    new_select = Some(selected);
                }
            }
        }

        let new_select = new_select.unwrap_or_else(|| get_near_index(&tmp_results, selected));

        let mut history = vec![];
        tmp_history.sort_by_key(|entry| std::cmp::Reverse(entry.num));

        for h in tmp_history.into_iter() {
            if h.num == 0 {
                history.insert(0, vec![h]);
            } else {
                history.push(vec![h]);
            }
        }

        self.history_area.reset_history_data(history);
        self.history_area.set_state_select(new_select);

        new_select
    }

    pub(super) fn create_result_items(
        &mut self,
        result: CommandResult,
        is_running_app: bool,
    ) -> bool {
        self.header_area.set_current_result(result.clone());
        self.header_area.update();

        let mut latest_result = CommandResult::default();
        if self.results.is_empty() {
            let init_items = ResultItems::default();

            self.results.insert(0, init_items.clone());
            self.results_stdout.insert(0, init_items.clone());
            self.results_stderr.insert(0, init_items.clone());
        } else {
            let latest_num = get_results_latest_index(&self.results);
            latest_result = self.results[&latest_num].command_result.clone();
        }

        if command_results_equivalent(&latest_result, &result, self.ignore_spaceblock) {
            return false;
        }

        let stdout_latest_index = get_results_latest_index(&self.results_stdout);
        let stdout_latest_result = self.results_stdout[&stdout_latest_index]
            .command_result
            .clone();

        let stderr_latest_index = get_results_latest_index(&self.results_stderr);
        let stderr_latest_result = self.results_stderr[&stderr_latest_index]
            .command_result
            .clone();

        let (output_result_items, stdout_result_items, stderr_result_items) = gen_result_items(
            result,
            self.summary_enabled,
            self.enable_summary_char,
            self.ignore_spaceblock,
            &latest_result,
            &stdout_latest_result,
            &stderr_latest_result,
        );

        let _ = self.update_result(
            output_result_items,
            stdout_result_items,
            stderr_result_items,
            is_running_app,
        );

        true
    }

    pub(super) fn update_result(
        &mut self,
        output_result_items: ResultItems,
        stdout_result_items: ResultItems,
        stderr_result_items: ResultItems,
        is_running_app: bool,
    ) -> bool {
        if self.results.is_empty() {
            self.results.insert(0, output_result_items.clone());
            self.results_stdout.insert(0, stdout_result_items.clone());
            self.results_stderr.insert(0, stderr_result_items.clone());
        }

        if !self.after_command.is_empty() && is_running_app {
            let after_command = self.after_command.clone();

            let results = self.results.clone();
            let latest_num = results.len() - 1;

            let before_result: CommandResult = self.results[&latest_num].command_result.clone();
            let after_result = output_result_items.command_result.clone();

            let after_command_result_write_file = self.after_command_result_write_file;
            let shell_command = self.after_command_shell_command.clone();

            thread::spawn(move || {
                exec_after_command(
                    shell_command,
                    after_command.clone(),
                    before_result,
                    after_result,
                    after_command_result_write_file,
                );
            });
        }

        let insert_result = self.insert_result(
            output_result_items,
            stdout_result_items,
            stderr_result_items,
        );
        let result_index = insert_result.0;
        let is_limit_over = insert_result.1;
        let is_update_stdout = insert_result.2;
        let is_update_stderr = insert_result.3;

        if !self.logfile.is_empty() && is_running_app {
            let _ = logging_result(&self.logfile, &self.results[&result_index].command_result);
        }

        let support_only_diffline: bool = self.diff_modes[self.diff_mode]
            .lock()
            .unwrap()
            .get_support_only_diffline();

        let mut is_push = true;
        if self.is_filtered {
            let result_text = match (
                self.output_mode,
                support_only_diffline,
                self.is_only_diffline,
            ) {
                (OutputMode::Output | OutputMode::Stdout | OutputMode::Stderr, true, true) => {
                    self.results[&result_index].get_diff_only_data(self.ansi_color)
                }
                (OutputMode::Output, _, _) => {
                    self.results[&result_index].command_result.get_output()
                }
                (OutputMode::Stdout, _, _) => {
                    self.results[&result_index].command_result.get_stdout()
                }
                (OutputMode::Stderr, _, _) => {
                    self.results[&result_index].command_result.get_stderr()
                }
            };

            is_push = self.matches_filter_text(&result_text);
        }

        let mut selected = self.history_area.get_state_select();
        if is_push {
            match self.output_mode {
                OutputMode::Output => self.add_history(result_index, selected),
                OutputMode::Stdout => {
                    if is_update_stdout {
                        self.add_history(result_index, selected)
                    }
                }
                OutputMode::Stderr => {
                    if is_update_stderr {
                        self.add_history(result_index, selected)
                    }
                }
            }
        }
        selected = self.history_area.get_state_select();

        if is_limit_over {
            self.reset_history(selected);
        }

        if is_running_app {
            self.set_output_data(selected);
        }

        true
    }

    pub(super) fn handle_exit_on_change(&mut self, changed: bool) {
        if self.exit_on_change.is_none() {
            return;
        }

        if !self.exit_on_change_armed {
            self.exit_on_change_armed = true;
            return;
        }

        if !changed {
            return;
        }

        if let Some(remaining) = self.exit_on_change.as_mut() {
            if *remaining > 0 {
                *remaining -= 1;
            }
            if *remaining == 0 {
                self.done = true;
            }
        }
    }

    pub(super) fn insert_result(
        &mut self,
        output_result_items: ResultItems,
        stdout_result_items: ResultItems,
        stderr_result_items: ResultItems,
    ) -> (usize, bool, bool, bool) {
        let result_index = self.results.keys().max().unwrap_or(&0) + 1;
        self.results.insert(result_index, output_result_items);

        let stdout_latest_index = get_results_latest_index(&self.results_stdout);
        let before_result_stdout = &self.results_stdout[&stdout_latest_index]
            .command_result
            .get_stdout();
        let result_stdout = &stdout_result_items.command_result.get_stdout();
        let mut is_stdout_update = false;
        if !text_eq_ignoring_space_blocks(
            before_result_stdout,
            result_stdout,
            self.ignore_spaceblock,
        ) {
            is_stdout_update = true;
            self.results_stdout
                .insert(result_index, stdout_result_items);
        }

        let stderr_latest_index = get_results_latest_index(&self.results_stderr);
        let before_result_stderr = &self.results_stderr[&stderr_latest_index]
            .command_result
            .get_stderr();
        let result_stderr = &stderr_result_items.command_result.get_stderr();
        let mut is_stderr_update = false;
        if !text_eq_ignoring_space_blocks(
            before_result_stderr,
            result_stderr,
            self.ignore_spaceblock,
        ) {
            is_stderr_update = true;
            self.results_stderr
                .insert(result_index, stderr_result_items);
        }

        let mut is_limit_over = false;
        if self.limit > 0 {
            let limit = self.limit as usize;
            if self.results.len() > limit {
                let mut keys: Vec<_> = self.results.keys().cloned().collect();
                keys.sort();

                let remove_count = self.results.len() - limit;

                for key in keys.iter().take(remove_count) {
                    self.results.remove(key);
                }

                is_limit_over = true;
            }

            if self.results_stdout.len() > limit {
                let mut keys: Vec<_> = self.results_stdout.keys().cloned().collect();
                keys.sort();

                let remove_count = self.results_stdout.len() - limit;

                for key in keys.iter().take(remove_count) {
                    self.results_stdout.remove(key);
                }

                is_limit_over = true;
            }

            if self.results_stderr.len() > limit {
                let mut keys: Vec<_> = self.results_stderr.keys().cloned().collect();
                keys.sort();

                let remove_count = self.results_stderr.len() - limit;

                for key in keys.iter().take(remove_count) {
                    self.results_stderr.remove(key);
                }

                is_limit_over = true;
            }
        }

        (
            result_index,
            is_limit_over,
            is_stdout_update,
            is_stderr_update,
        )
    }
}

pub(super) fn get_near_index(results: &HashMap<usize, ResultItems>, index: usize) -> usize {
    let keys = results.keys().cloned().collect::<Vec<usize>>();

    if keys.contains(&index) {
        index
    } else if index > get_results_latest_index(results) {
        get_results_next_index(results, index)
    } else {
        get_results_previous_index(results, index)
    }
}

pub(super) fn get_results_latest_index(results: &HashMap<usize, ResultItems>) -> usize {
    let mut result_num = 0;
    let mut keys = results.keys().cloned().collect::<Vec<usize>>();
    keys.sort();

    let latest_num = keys.iter().last();
    if let Some(number) = latest_num {
        result_num = *number
    }

    result_num
}

pub(super) fn get_results_previous_index(
    results: &HashMap<usize, ResultItems>,
    index: usize,
) -> usize {
    let mut result_num = 0;
    let mut keys = results.keys().cloned().collect::<Vec<usize>>();
    keys.sort();

    for key in keys {
        if index > key {
            result_num = key;
        }
    }

    result_num
}

pub(super) fn get_results_next_index(results: &HashMap<usize, ResultItems>, index: usize) -> usize {
    let mut result_num = 0;
    let mut keys = results.keys().cloned().collect::<Vec<usize>>();
    keys.sort();

    for key in keys {
        if index < key {
            result_num = key;
            break;
        }
    }

    result_num
}

pub(super) fn retain_selected_and_latest_result_only(
    results: &mut HashMap<usize, ResultItems>,
    selected: usize,
) {
    if results.is_empty() {
        return;
    }

    let latest = get_results_latest_index(results);
    results.retain(|k, _| *k == 0 || *k == latest || *k == selected);
}

pub(super) fn gen_result_items(
    result: CommandResult,
    summary_enabled: bool,
    enable_summary_char: bool,
    ignore_spaceblock: bool,
    output_latest_result: &CommandResult,
    stdout_latest_result: &CommandResult,
    stderr_latest_result: &CommandResult,
) -> (ResultItems, ResultItems, ResultItems) {
    let output_diff_only_data = gen_diff_only_data(
        &output_latest_result.get_output(),
        &result.get_output(),
        ignore_spaceblock,
    );
    let mut output_result_items = ResultItems {
        command_result: result.clone(),
        summary: HistorySummary::init(),
        diff_only_data: output_diff_only_data,
    };
    if summary_enabled {
        output_result_items.summary.calc(
            &output_latest_result.get_output(),
            &output_result_items.command_result.get_output(),
            enable_summary_char,
            ignore_spaceblock,
        );
    }

    let stdout_diff_only_data = gen_diff_only_data(
        &stdout_latest_result.get_stdout(),
        &result.get_stdout(),
        ignore_spaceblock,
    );
    let mut stdout_result_items = ResultItems {
        command_result: result.clone(),
        summary: HistorySummary::init(),
        diff_only_data: stdout_diff_only_data,
    };
    if summary_enabled {
        stdout_result_items.summary.calc(
            &stdout_latest_result.get_stdout(),
            &stdout_result_items.command_result.get_stdout(),
            enable_summary_char,
            ignore_spaceblock,
        );
    }

    let stderr_diff_only_data = gen_diff_only_data(
        &stderr_latest_result.get_stderr(),
        &result.get_stderr(),
        ignore_spaceblock,
    );
    let mut stderr_result_items = ResultItems {
        command_result: result.clone(),
        summary: HistorySummary::init(),
        diff_only_data: stderr_diff_only_data,
    };
    if summary_enabled {
        stderr_result_items.summary.calc(
            &stderr_latest_result.get_stderr(),
            &stderr_result_items.command_result.get_stderr(),
            enable_summary_char,
            ignore_spaceblock,
        );
    }

    (
        output_result_items,
        stdout_result_items,
        stderr_result_items,
    )
}

pub(super) fn command_results_equivalent(
    before: &CommandResult,
    after: &CommandResult,
    ignore_spaceblock: bool,
) -> bool {
    before.command == after.command
        && before.status == after.status
        && text_eq_ignoring_space_blocks(
            &before.get_output(),
            &after.get_output(),
            ignore_spaceblock,
        )
        && text_eq_ignoring_space_blocks(
            &before.get_stdout(),
            &after.get_stdout(),
            ignore_spaceblock,
        )
        && text_eq_ignoring_space_blocks(
            &before.get_stderr(),
            &after.get_stderr(),
            ignore_spaceblock,
        )
}

pub(super) fn gen_diff_only_data(before: &str, after: &str, ignore_spaceblock: bool) -> Vec<u8> {
    let mut diff_only_data = vec![];

    let before = if ignore_spaceblock {
        hwatch_diffmode::normalize_space_blocks(before)
    } else {
        before.to_string()
    };
    let after = if ignore_spaceblock {
        hwatch_diffmode::normalize_space_blocks(after)
    } else {
        after.to_string()
    };

    let diff_set = TextDiff::from_lines(&before, &after);
    for op in diff_set.ops() {
        for change in diff_set.iter_changes(op) {
            match change.tag() {
                ChangeTag::Equal => {}
                ChangeTag::Delete | ChangeTag::Insert => {
                    let value = change.to_string().as_bytes().to_vec();
                    diff_only_data.extend(value);
                }
            }
        }
    }

    diff_only_data
}
