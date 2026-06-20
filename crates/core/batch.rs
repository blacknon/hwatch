// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crossbeam_channel::Receiver;
use std::thread;
use std::{collections::HashMap, io};

use crate::common::{logging_result, OutputMode};
use crate::event::AppEvent;
use crate::exec::{exec_after_command, CommandResult};
use crate::output;
use crate::DiffModeRef;
use crate::DiffModeRef;

use hwatch_diffmode::text_eq_ignoring_space_blocks;

struct DecodedCommandResult {
    output: String,
    stdout: String,
    stderr: String,
}

impl DecodedCommandResult {
    fn from_result(result: &CommandResult) -> Self {
        Self {
            output: result.get_output(),
            stdout: result.get_stdout(),
            stderr: result.get_stderr(),
        }
    }
}

/// Struct at watch view window.
pub struct Batch {
    // Command executed after a change is detected.
    // Command executed after a change is detected.
    after_command: String,

    // Shell used to execute the after-command hook.
    // Shell used to execute the after-command hook.
    after_command_shell_command: String,

    // Whether hook payloads are written to a temp file first.
    // Whether hook payloads are written to a temp file first.
    after_command_result_write_file: bool,

    // Whether to print line numbers in diff output.
    // Whether to print line numbers in diff output.
    line_number: bool,

    // Whether ANSI color output should be preserved.
    // Whether ANSI color output should be preserved.
    is_color: bool,

    // Whether to beep when output changes.
    // Whether to beep when output changes.
    is_beep: bool,

    // Remaining change count before exiting, if enabled.
    // Remaining change count before exiting, if enabled.
    exit_on_change: Option<u32>,

    // Prevents the first observed result from counting as a change.
    // Prevents the first observed result from counting as a change.
    exit_on_change_armed: bool,

    // Whether batch output should be reversed.
    // Whether batch output should be reversed.
    is_reverse: bool,

    // Accumulated command history for batch mode.
    // Accumulated command history for batch mode.
    results: HashMap<usize, CommandResult>,

    // Selected output stream to compare and print.
    // Selected output stream to compare and print.
    output_mode: OutputMode,

    // Selected diff mode index.
    // Selected diff mode index.
    diff_mode: usize,

    // Available diff mode implementations.
    diff_modes: Vec<DiffModeRef>,
    // Available diff mode implementations.
    diff_modes: Vec<DiffModeRef>,

    // Whether only changed lines should be shown.
    // Whether only changed lines should be shown.
    is_only_diffline: bool,

    // Whether whitespace-only block changes should be ignored.
    // Whether whitespace-only block changes should be ignored.
    ignore_spaceblock: bool,

    // Optional logfile path for persisted history.
    // Optional logfile path for persisted history.
    logfile: String,

    // Shared rendering helper for batch output.
    // Shared rendering helper for batch output.
    printer: output::Printer,

    // Receives command execution events from the runner thread.
    // Receives command execution events from the runner thread.
    pub rx: Receiver<AppEvent>,
}

impl Batch {
    // Creates batch-mode application state.
    pub fn new(rx: Receiver<AppEvent>, diff_modes: Vec<DiffModeRef>) -> Self {
    // Creates batch-mode application state.
    pub fn new(rx: Receiver<AppEvent>, diff_modes: Vec<DiffModeRef>) -> Self {
        // Create Default DiffMode
        let diff_mode_counter = 0;
        let diff_mode_ref = diff_modes[diff_mode_counter].clone();
        let diff_mode_ref = diff_modes[diff_mode_counter].clone();

        Self {
            after_command: "".to_string(),
            after_command_shell_command: crate::SHELL_COMMAND.to_string(),
            after_command_result_write_file: false,
            line_number: false,
            is_color: true,
            is_beep: false,
            exit_on_change: None,
            exit_on_change_armed: false,
            is_reverse: false,
            results: HashMap::new(),
            output_mode: OutputMode::Output,
            diff_mode: 0,
            diff_modes,
            is_only_diffline: false,
            ignore_spaceblock: false,
            logfile: "".to_string(),
            printer: output::Printer::new(diff_mode_ref),
            printer: output::Printer::new(diff_mode_ref),
            rx,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.printer
            .set_batch(true)
            .set_color(self.is_color)
            .set_diff_mode(self.diff_modes[self.diff_mode].clone())
            .set_line_number(self.line_number)
            .set_reverse(self.is_reverse)
            .set_only_diffline(self.is_only_diffline)
            .set_ignore_spaceblock(self.ignore_spaceblock)
            .set_output_mode(self.output_mode);

        loop {
            if matches!(self.exit_on_change, Some(0)) {
                return Ok(());
            }
            match self.rx.recv() {
                // Get command result.
                Ok(AppEvent::OutputUpdate(exec_result)) => {
                    let changed = self.update_result(exec_result);

                    // beep
                    if changed && self.is_beep {
                        println!("\x07")
                    }

                    if self.handle_exit_on_change(changed) {
                        return Ok(());
                    }
                }

                // Other event
                Ok(_) => {}

                // Error
                Err(_) => {}
            }
        }
    }

    // Processes a new command result and prints when the selected stream changes.
    // Processes a new command result and prints when the selected stream changes.
    fn update_result(&mut self, _result: CommandResult) -> bool {
        // check results size.
        let mut latest_result = CommandResult::default();

        if self.results.is_empty() {
            // diff output data.
            self.results.insert(0, latest_result.clone());
        } else {
            let latest_num = self.results.len() - 1;
            latest_result = self.results[&latest_num].clone();
        }

        // check result diff
        // NOTE: ここで実行結果の差分を比較している // 0.3.12リリースしたら消す
        if command_results_equivalent(&latest_result, &_result, self.ignore_spaceblock) {
            return false;
        }

        // logging result.
        if !self.logfile.is_empty() {
            let _ = logging_result(&self.logfile, &_result);
        }

        if !self.after_command.is_empty() {
            let after_command = self.after_command.clone();
            let latest_num = self.results.len() - 1;
            let before_result = self.results[&latest_num].clone();
            let after_result = _result.clone();
            let after_command_result_write_file = self.after_command_result_write_file;
            let shell_command = self.after_command_shell_command.clone();

            {
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
        }

        let should_print = self.should_print_for_output_mode(&latest_result, &_result);

        // add result
        self.results.insert(self.results.len(), _result.clone());

        // output result
        if should_print {
            self.printout_result();
        }

        true
    }

    fn should_print_for_output_mode(&self, before: &CommandResult, after: &CommandResult) -> bool {
        let before = DecodedCommandResult::from_result(before);
        let after = DecodedCommandResult::from_result(after);

        match self.output_mode {
            OutputMode::Output => !text_eq_ignoring_space_blocks(
                &before.output,
                &after.output,
                self.ignore_spaceblock,
            ),
            OutputMode::Stdout => !text_eq_ignoring_space_blocks(
                &before.stdout,
                &after.stdout,
                self.ignore_spaceblock,
            ),
            OutputMode::Stderr => !text_eq_ignoring_space_blocks(
                &before.stderr,
                &after.stderr,
                self.ignore_spaceblock,
            ),
        }
    }

    // Prints the currently selected diff to stdout.
    fn printout_result(&mut self) {
        // output result
        let latest = self.results.len() - 1;

        // Switch the result depending on the output mode.
        let dest = &self.results[&latest];
        let timestamp_dst = &dest.timestamp;

        let previous = latest - 1;
        let src = &self.results[&previous];

        // print split line
        if self.is_color {
            println!(
                "\x1b[38;5;240m=====[{:}]=========================\x1b[0m",
                timestamp_dst
            );
        } else {
            println!("=====[{:}]=========================", timestamp_dst);
        }

        let printout_data = self.printer.get_batch_text(dest, src);

        if printout_data.is_empty() {
            return;
        }

        println!("{:}", printout_data.join("\n"));
    }

    // Sets the after-command hook.
    pub fn set_after_command(mut self, after_command: String) -> Self {
        self.after_command = after_command;
        self
    }

    pub fn set_after_command_shell_command(mut self, shell_command: String) -> Self {
        self.after_command_shell_command = shell_command;
        self
    }

    pub fn set_after_command_result_write_file(mut self, write_file: bool) -> Self {
        self.after_command_result_write_file = write_file;
        self
    }

    // Enables or disables line numbers in batch output.
    pub fn set_line_number(mut self, line_number: bool) -> Self {
        self.line_number = line_number;
        self
    }

    // Enables or disables terminal beeps on changes.
    pub fn set_beep(mut self, is_beep: bool) -> Self {
        self.is_beep = is_beep;
        self
    }

    // Enables or disables ANSI color output.
    pub fn set_color(mut self, is_color: bool) -> Self {
        self.is_color = is_color;
        self
    }

    // Configures change-count based exit behavior.
    pub fn set_exit_on_change(mut self, exit_on_change: Option<u32>) -> Self {
        self.exit_on_change = exit_on_change;
        self.exit_on_change_armed = false;
        self
    }

    // Enables or disables reverse output order.
    pub fn set_reverse(mut self, is_reverse: bool) -> Self {
        self.is_reverse = is_reverse;
        self
    }

    // Selects the stream shown in batch mode.
    pub fn set_output_mode(mut self, output_mode: OutputMode) -> Self {
        self.output_mode = output_mode;
        self
    }

    // Selects the active diff mode by index.
    pub fn set_diff_mode(mut self, diff_mode: usize) -> Self {
        self.diff_mode = diff_mode;
        self
    }

    // Enables or disables diff-only output.
    pub fn set_only_diffline(mut self, is_only_diffline: bool) -> Self {
        self.is_only_diffline = is_only_diffline;
        self
    }

    // Enables or disables whitespace-block normalization in comparisons.
    // Enables or disables whitespace-block normalization in comparisons.
    pub fn set_ignore_spaceblock(mut self, ignore_spaceblock: bool) -> Self {
        self.ignore_spaceblock = ignore_spaceblock;
        self
    }

    // Sets the logfile path used for persisted history.
    // Sets the logfile path used for persisted history.
    pub fn set_logfile(mut self, logfile: String) -> Self {
        self.logfile = logfile;
        self
    }

    // Handles deferred exit-on-change activation and countdown.
    fn handle_exit_on_change(&mut self, changed: bool) -> bool {
        if self.exit_on_change.is_none() {
            return false;
        }

        if !self.exit_on_change_armed {
            self.exit_on_change_armed = true;
            return false;
        }

        if !changed {
            return false;
        }

        if let Some(remaining) = self.exit_on_change.as_mut() {
            if *remaining > 0 {
                *remaining -= 1;
            }
            *remaining == 0
        } else {
            false
            *remaining == 0
        } else {
            false
        }
    }
}

fn command_results_equivalent(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{load_logfile, OutputMode};
    use crate::diffmode_plane::DiffModeAtPlane;
    use crossbeam_channel::unbounded;
    use tempfile::NamedTempFile;

    #[cfg(not(skip_proptest_tests))]
    use proptest::prelude::*;

    fn new_batch(output_mode: OutputMode) -> Batch {
        let (_tx, rx) = unbounded();
        let diff_modes: Vec<DiffModeRef> = vec![std::rc::Rc::new(std::cell::RefCell::new(
            Box::new(DiffModeAtPlane::new()),
        ))];
        let diff_modes: Vec<DiffModeRef> = vec![std::rc::Rc::new(std::cell::RefCell::new(
            Box::new(DiffModeAtPlane::new()),
        ))];

        Batch::new(rx, diff_modes).set_output_mode(output_mode)
    }

    #[test]
    fn should_print_for_output_mode_uses_stdout_diff_only() {
        let batch = new_batch(OutputMode::Stdout);
        let before = CommandResult::default()
            .set_output(b"same stdout\nstderr-1\n".to_vec())
            .set_stdout(b"same stdout\n".to_vec())
            .set_stderr(b"stderr-1\n".to_vec());
        let after = CommandResult::default()
            .set_output(b"same stdout\nstderr-2\n".to_vec())
            .set_stdout(b"same stdout\n".to_vec())
            .set_stderr(b"stderr-2\n".to_vec());

        assert!(!batch.should_print_for_output_mode(&before, &after));
    }

    #[test]
    fn should_print_for_output_mode_uses_stderr_diff_only() {
        let batch = new_batch(OutputMode::Stderr);
        let before = CommandResult::default()
            .set_output(b"stdout-1\nsame stderr\n".to_vec())
            .set_stdout(b"stdout-1\n".to_vec())
            .set_stderr(b"same stderr\n".to_vec());
        let after = CommandResult::default()
            .set_output(b"stdout-2\nsame stderr\n".to_vec())
            .set_stdout(b"stdout-2\n".to_vec())
            .set_stderr(b"same stderr\n".to_vec());

        assert!(!batch.should_print_for_output_mode(&before, &after));
    }

    #[test]
    fn should_print_for_output_mode_detects_selected_stream_changes() {
        let stdout_batch = new_batch(OutputMode::Stdout);
        let stderr_batch = new_batch(OutputMode::Stderr);
        let before = CommandResult::default()
            .set_output(b"stdout-1\nstderr-1\n".to_vec())
            .set_stdout(b"stdout-1\n".to_vec())
            .set_stderr(b"stderr-1\n".to_vec());
        let after = CommandResult::default()
            .set_output(b"stdout-2\nstderr-2\n".to_vec())
            .set_stdout(b"stdout-2\n".to_vec())
            .set_stderr(b"stderr-2\n".to_vec());

        assert!(stdout_batch.should_print_for_output_mode(&before, &after));
        assert!(stderr_batch.should_print_for_output_mode(&before, &after));
    }

    #[test]
    fn should_print_for_output_mode_uses_combined_output_when_selected() {
        let batch = new_batch(OutputMode::Output);
        let before = CommandResult::default()
            .set_output(b"same stdout\nstderr-1\n".to_vec())
            .set_stdout(b"same stdout\n".to_vec())
            .set_stderr(b"stderr-1\n".to_vec());
        let after = CommandResult::default()
            .set_output(b"same stdout\nstderr-2\n".to_vec())
            .set_stdout(b"same stdout\n".to_vec())
            .set_stderr(b"stderr-2\n".to_vec());

        assert!(batch.should_print_for_output_mode(&before, &after));
    }

    #[test]
    fn should_print_for_output_mode_ignores_space_blocks_when_enabled() {
        let batch = new_batch(OutputMode::Stdout).set_ignore_spaceblock(true);
        let before = CommandResult::default()
            .set_output(b"alpha  beta\n".to_vec())
            .set_stdout(b"alpha  beta\n".to_vec())
            .set_stderr(b"".to_vec());
        let after = CommandResult::default()
            .set_output(b"alpha   beta\n".to_vec())
            .set_stdout(b"alpha   beta\n".to_vec())
            .set_stderr(b"".to_vec());

        assert!(!batch.should_print_for_output_mode(&before, &after));
    }

    #[test]
    fn command_results_equivalent_detects_command_and_status_changes() {
        let base = CommandResult::default()
            .set_output(b"same\n".to_vec())
            .set_stdout(b"same\n".to_vec())
            .set_stderr(b"".to_vec());
        let command_changed = CommandResult {
            command: "different".to_string(),
            ..base.clone()
        };
        let status_changed = CommandResult {
            status: false,
            ..base.clone()
        };

        assert!(!command_results_equivalent(&base, &command_changed, false));
        assert!(!command_results_equivalent(&base, &status_changed, false));
    }

    #[test]
    fn command_results_equivalent_ignores_space_blocks_when_enabled() {
        let before = CommandResult::default()
            .set_output(b"same\n".to_vec())
            .set_stdout(b"alpha  beta\n".to_vec())
            .set_stderr(b"stderr\n".to_vec());
        let after = CommandResult::default()
            .set_output(b"same\n".to_vec())
            .set_stdout(b"alpha   beta\n".to_vec())
            .set_stderr(b"stderr\n".to_vec());

        assert!(command_results_equivalent(&before, &after, true));
        assert!(!command_results_equivalent(&before, &after, false));
    }

    #[test]
    fn update_result_logs_current_result_instead_of_previous_one() {
        let logfile = NamedTempFile::new().unwrap();
        let path = logfile.path().to_string_lossy().into_owned();
        let mut batch = new_batch(OutputMode::Output).set_logfile(path.clone());
        let result = CommandResult {
            timestamp: "2026-04-24 21:30:00.000".to_string(),
            command: "echo current".to_string(),
            status: true,
            ..CommandResult::default()
        }
        .set_output(b"current\n".to_vec())
        .set_stdout(b"current\n".to_vec());

        assert!(batch.update_result(result.clone()));

        let loaded = load_logfile(&path, false);
        assert!(loaded.is_ok());
        let loaded = loaded.ok().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded[0] == result);
    }

    #[cfg(not(skip_proptest_tests))]
    proptest! {
        #[test]
        fn command_results_equivalent_is_reflexive(
            command in "[^\0]{0,64}",
            output in "[^\0]{0,64}",
            stdout in "[^\0]{0,64}",
            stderr in "[^\0]{0,64}",
            status in any::<bool>(),
        ) {
            let result = CommandResult {
                command,
                status,
                ..CommandResult::default()
            }
            .set_output(output.as_bytes().to_vec())
            .set_stdout(stdout.as_bytes().to_vec())
            .set_stderr(stderr.as_bytes().to_vec());

            prop_assert!(command_results_equivalent(&result, &result, false));
            prop_assert!(command_results_equivalent(&result, &result, true));
        }

        #[test]
        fn command_results_equivalent_ignores_normalized_space_only_diffs(
            left in "[^\n\r]{0,32}",
            spaces_a in "[ \t]{1,8}",
            spaces_b in "[ \t]{1,8}",
            right in "[^\n\r]{0,32}",
        ) {
            let before_stdout = format!("{left}{spaces_a}{right}\n");
            let after_stdout = format!("{left}{spaces_b}{right}\n");

            prop_assume!(hwatch_diffmode::normalize_space_blocks(&before_stdout)
                == hwatch_diffmode::normalize_space_blocks(&after_stdout));

            let before = CommandResult::default()
                .set_output(before_stdout.as_bytes().to_vec())
                .set_stdout(before_stdout.as_bytes().to_vec())
                .set_stderr(b"stderr\n".to_vec());
            let after = CommandResult::default()
                .set_output(after_stdout.as_bytes().to_vec())
                .set_stdout(after_stdout.as_bytes().to_vec())
                .set_stderr(b"stderr\n".to_vec());

            prop_assert!(command_results_equivalent(&before, &after, true));
        }
    }
}
