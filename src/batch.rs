// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crossbeam_channel::{Receiver, Sender};
use std::{io, collections::HashMap};
use std::thread;

use crate::common::{DiffMode, OutputMode};
use crate::event::AppEvent;
use crate::exec::{exec_after_command, CommandResult};
use crate::output;

/// Struct at watch view window.
pub struct Batch {
    ///
    after_command: String,

    ///
    line_number: bool,

    ///
    is_color: bool,

    ///
    is_beep: bool,

    ///
    is_reverse: bool,

    ///
    results: HashMap<usize, CommandResult>,

    ///
    output_mode: OutputMode,

    ///
    diff_mode: DiffMode,

    ///
    is_only_diffline: bool,

    ///
    printer: output::Printer,

    ///
    pub tx: Sender<AppEvent>,

    ///
    pub rx: Receiver<AppEvent>,
}

impl Batch {
    ///
    pub fn new(tx: Sender<AppEvent>, rx: Receiver<AppEvent>) -> Self {
        Self {
            after_command: "".to_string(),
            line_number: false,
            is_color: true,
            is_beep: false,
            is_reverse: false,
            results: HashMap::new(),
            output_mode: OutputMode::Output,
            diff_mode: DiffMode::Disable,
            is_only_diffline: false,
            printer: output::Printer::new(),
            tx,
            rx,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.printer
            .set_batch(true)
            .set_color(self.is_color)
            .set_diff_mode(self.diff_mode)
            .set_line_number(self.line_number)
            .set_reverse(self.is_reverse)
            .set_only_diffline(self.is_only_diffline)
            .set_output_mode(self.output_mode);

        loop {
            match self.rx.recv() {
                // Get command result.
                Ok(AppEvent::OutputUpdate(exec_result)) => {
                    let _exec_return = self.update_result(exec_result);

                    // beep
                    if _exec_return && self.is_beep {
                        println!("\x07")
                    }
                },

                // Other event
                Ok(_) => {},

                // Error
                Err(_) => {}
            }
        }
    }

    ///
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
        if latest_result == _result {
            return false;
        }

        if !self.after_command.is_empty() {
            let after_command = self.after_command.clone();

            let results = self.results.clone();
            let latest_num = results.len() - 1;

            let before_result = results[&latest_num].clone();
            let after_result = _result.clone();

            {
                thread::spawn(move || {
                    exec_after_command(
                        "sh -c".to_string(),
                        after_command.clone(),
                        before_result,
                        after_result,
                    );
                });
            }
        }

        // add result
        self.results.insert(self.results.len(), _result.clone());

        // output result
        self.printout_result();

        true
    }

    ///
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
            println!("\x1b[38;5;240m=====[{:}]=========================\x1b[0m", timestamp_dst);
        } else {
            println!("=====[{:}]=========================", timestamp_dst);
        }

        let printout_data = self.printer.get_batch_text(dest, src);

        println!("{:}", printout_data.join("\n"));
    }

    ///
    pub fn set_after_command(mut self, after_command: String) -> Self {
        self.after_command = after_command;
        self
    }

    ///
    pub fn set_line_number(mut self, line_number: bool) -> Self {
        self.line_number = line_number;
        self
    }

    ///
    pub fn set_beep(mut self, is_beep: bool) -> Self {
        self.is_beep = is_beep;
        self
    }

    ///
    pub fn set_reverse(mut self, is_reverse: bool) -> Self {
        self.is_reverse = is_reverse;
        self
    }

    ///
    pub fn set_output_mode(mut self, output_mode: OutputMode) -> Self {
        self.output_mode = output_mode;
        self
    }

    ///
    pub fn set_diff_mode(mut self, diff_mode: DiffMode) -> Self {
        self.diff_mode = diff_mode;
        self
    }

    ///
    pub fn set_only_diffline(mut self, is_only_diffline: bool) -> Self {
        self.is_only_diffline = is_only_diffline;
        self
    }
}
