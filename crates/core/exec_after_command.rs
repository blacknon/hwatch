// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use serde::Serialize;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

use super::{create_exec_cmd_args, CommandResult, CommandResultData};

#[derive(Serialize)]
pub struct ExecuteAfterResultData {
    pub before_result: CommandResultData,
    pub after_result: CommandResultData,
}

pub fn exec_after_command(
    shell_command: String,
    after_command: String,
    before: CommandResult,
    after: CommandResult,
    after_command_result_write_file: bool,
) {
    let before_result: CommandResultData = before.export_data();
    let after_result = after.export_data();

    let result_data = ExecuteAfterResultData {
        before_result,
        after_result,
    };

    let json_data: String = match serde_json::to_string(&result_data) {
        Ok(json_data) => json_data,
        Err(err) => {
            eprintln!("Failed to serialize after command payload: {err}");
            return;
        }
    };

    let exec_commands = match create_exec_cmd_args(false, shell_command, after_command) {
        Ok(exec_commands) => exec_commands,
        Err(err) => {
            eprintln!("Failed to prepare after command: {err}");
            return;
        }
    };
    let length = exec_commands.len();

    if after_command_result_write_file {
        let mut file = match NamedTempFile::new() {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to create temporary file for after command result: {err}");
                return;
            }
        };

        if let Err(err) = file.write_all(json_data.as_bytes()) {
            eprintln!("Failed to write after command payload to file: {err}");
            return;
        }
        if let Err(err) = file.flush() {
            eprintln!("Failed to flush after command payload file: {err}");
            return;
        }

        let hwatch_result_data = file.path().to_string_lossy().into_owned();

        let child = Command::new(&exec_commands[0])
            .args(&exec_commands[1..length])
            .env("HWATCH_DATA", hwatch_result_data)
            .spawn();

        match child {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(err) => {
                eprintln!("Failed to execute after command: {err}");
            }
        }

        if let Err(err) = file.close() {
            eprintln!("Failed to close temporary file for after command result: {err}");
        }
    } else {
        let hwatch_result_data = json_data;

        let child = Command::new(&exec_commands[0])
            .args(&exec_commands[1..length])
            .env("HWATCH_DATA", hwatch_result_data)
            .spawn();

        match child {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(err) => {
                eprintln!("Failed to execute after command: {err}");
            }
        }
    }
}
