// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): outputやcommandの型をbyteに変更する
// TODO(blacknon): `command`は別のトコで保持するように変更する？(メモリの節約のため)

// module
use crossbeam_channel::Sender;
use flate2::{read::GzDecoder, write::GzEncoder};
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

// local module
use crate::common;
use crate::common::OutputMode;
use crate::event::AppEvent;

// struct for handling data during log/after exec
#[derive(Serialize, Deserialize)]
pub struct CommandResultData {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
}

impl CommandResultData {
    pub fn generate_result(&self, is_compress: bool) -> CommandResult {
        let output = self.output.as_bytes().to_vec();
        let stdout = self.stdout.as_bytes().to_vec();
        let stderr = self.stderr.as_bytes().to_vec();

        CommandResult {
            timestamp: self.timestamp.clone(),
            command: self.command.clone(),
            status: self.status,
            is_compress,
            output: vec![],
            stdout: vec![],
            stderr: vec![],
        }
        .set_output(output)
        .set_stdout(stdout)
        .set_stderr(stderr)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CommandResult {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub is_compress: bool,
    pub output: Vec<u8>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            timestamp: String::default(),
            command: String::default(),
            status: true,
            is_compress: false,
            output: vec![],
            stdout: vec![],
            stderr: vec![],
        }
    }
}

impl PartialEq for CommandResult {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.status == other.status
            && self.output == other.output
            && self.stdout == other.stdout
            && self.stderr == other.stderr
    }
}

impl CommandResult {
    fn set_data(&self, data: Vec<u8>, data_type: OutputMode) -> Self {
        let u8_data = if self.is_compress {
            let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&data).unwrap();
            encoder.finish().unwrap()
        } else {
            data
        };

        match data_type {
            OutputMode::Output => CommandResult {
                output: u8_data,
                ..self.clone()
            },
            OutputMode::Stdout => CommandResult {
                stdout: u8_data,
                ..self.clone()
            },
            OutputMode::Stderr => CommandResult {
                stderr: u8_data,
                ..self.clone()
            },
        }
    }

    pub fn set_output(&self, data: Vec<u8>) -> Self {
        self.set_data(data, OutputMode::Output)
    }

    pub fn set_stdout(&self, data: Vec<u8>) -> Self {
        self.set_data(data, OutputMode::Stdout)
    }

    pub fn set_stderr(&self, data: Vec<u8>) -> Self {
        self.set_data(data, OutputMode::Stderr)
    }

    fn get_data(&self, data_type: OutputMode) -> String {
        let data = match data_type {
            OutputMode::Output => &self.output,
            OutputMode::Stdout => &self.stdout,
            OutputMode::Stderr => &self.stderr,
        };

        if self.is_compress {
            let mut decoder = GzDecoder::new(&data[..]);
            let mut s = String::new();
            decoder.read_to_string(&mut s).unwrap();
            s
        } else {
            String::from_utf8_lossy(data).to_string()
        }
    }

    pub fn get_output(&self) -> String {
        self.get_data(OutputMode::Output)
    }

    pub fn get_stdout(&self) -> String {
        self.get_data(OutputMode::Stdout)
    }

    pub fn get_stderr(&self) -> String {
        self.get_data(OutputMode::Stderr)
    }

    pub fn export_data(&self) -> CommandResultData {
        CommandResultData {
            timestamp: self.timestamp.clone(),
            command: self.command.clone(),
            status: self.status,
            output: self.get_output(),
            stdout: self.get_stdout(),
            stderr: self.get_stderr(),
        }
    }
}

// TODO(blacknon): commandは削除？
pub struct ExecuteCommand {
    pub shell_command: String,
    pub command: Vec<String>,
    pub is_exec: bool,
    pub is_compress: bool,
    pub output_width: Option<usize>,
    pub tx: Sender<AppEvent>,
}

impl ExecuteCommand {
    // set default value
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self {
            shell_command: "".to_string(),
            command: vec![],
            is_exec: false,
            is_compress: false,
            output_width: None,
            tx,
        }
    }

    // exec command
    // TODO(blacknon): Resultからcommandを削除して、実行時はこのfunctionの引数として受け付けるように改修する？
    pub fn exec_command(&mut self) {
        // set string command.
        let command_str = self.command.clone().join(" ");

        // create exec_commands...
        let exec_commands = create_exec_cmd_args(
            self.is_exec,
            self.shell_command.clone(),
            command_str.clone(),
        );

        // exec command...
        let length = exec_commands.len();
        let mut child_command = Command::new(&exec_commands[0]);
        child_command
            .args(&exec_commands[1..length])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(cols) = self.output_width {
            child_command
                .env("COLUMNS", cols.to_string())
                .env("HWATCH_COLUMNS", cols.to_string());
        }
        let child_result = child_command.spawn();

        // merge stdout and stderr
        let mut vec_output = Vec::new();
        let mut vec_stdout = Vec::new();
        let mut vec_stderr = Vec::new();

        // get output data
        let status = match child_result {
            Ok(mut child) => {
                let child_stdout = child.stdout.take().expect("");
                let child_stderr = child.stderr.take().expect("");

                // Prepare vector for collapsing stdout and stderr output
                let arc_vec_output = Arc::new(Mutex::new(Vec::new()));
                let arc_vec_stdout = Arc::new(Mutex::new(Vec::new()));
                let arc_vec_stderr = Arc::new(Mutex::new(Vec::new()));

                // Using Arc and Mutex to share sequential writes between threads
                let arc_vec_output_stdout_clone = Arc::clone(&arc_vec_output);
                let arc_vec_output_stderr_clone = Arc::clone(&arc_vec_output);
                let arc_vec_stdout_clone = Arc::clone(&arc_vec_stdout);
                let arc_vec_stderr_clone = Arc::clone(&arc_vec_stderr);

                // start stdout thread
                let stdout_thread = thread::spawn(move || {
                    // Use BufReader to read child process stdout
                    let mut stdout = BufReader::new(child_stdout);
                    let mut buf = Vec::new();

                    // write to vector
                    stdout.read_to_end(&mut buf).expect("Failed to read stdout");
                    arc_vec_stdout_clone.lock().unwrap().extend_from_slice(&buf);
                    arc_vec_output_stdout_clone
                        .lock()
                        .unwrap()
                        .extend_from_slice(&buf);
                });

                // start stderr thread
                let stderr_thread = thread::spawn(move || {
                    // Use BufReader to read child process stderr
                    let mut stderr = BufReader::new(child_stderr);
                    let mut buf = Vec::new();

                    // write to vector
                    stderr.read_to_end(&mut buf).expect("Failed to read stderr");
                    arc_vec_stderr_clone.lock().unwrap().extend_from_slice(&buf);
                    arc_vec_output_stderr_clone
                        .lock()
                        .unwrap()
                        .extend_from_slice(&buf);
                });

                // with thread stdout/stderr
                stdout_thread.join().expect("Failed to join stdout thread");
                stderr_thread.join().expect("Failed to join stderr thread");

                // Unwrap Arc, get MutexGuard and extract vector
                vec_output = Arc::try_unwrap(arc_vec_output)
                    .unwrap()
                    .into_inner()
                    .unwrap();
                vec_stdout = Arc::try_unwrap(arc_vec_stdout)
                    .unwrap()
                    .into_inner()
                    .unwrap();
                vec_stderr = Arc::try_unwrap(arc_vec_stderr)
                    .unwrap()
                    .into_inner()
                    .unwrap();

                // get command status
                let exit_status = child.wait().expect("");
                exit_status.success()
            }
            Err(err) => {
                let error_msg = err.to_string();

                let mut stdout_text: Vec<u8> = error_msg.as_bytes().to_vec();
                let mut stderr_text: Vec<u8> = error_msg.as_bytes().to_vec();
                vec_output.append(&mut stdout_text);
                vec_stderr.append(&mut stderr_text);

                // get command status
                false
            }
        };

        // Set result
        let result = CommandResult {
            timestamp: common::now_str(),
            command: command_str,
            status,
            is_compress: self.is_compress,
            output: vec![],
            stdout: vec![],
            stderr: vec![],
        }
        .set_output(vec_output)
        .set_stdout(vec_stdout)
        .set_stderr(vec_stderr);

        // Send result
        let _ = self.tx.send(AppEvent::OutputUpdate(result));
    }
}

// TODO: 変化が発生した時の後処理コマンドを実行するためのstruct
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
) {
    let before_result: CommandResultData = before.export_data();
    let after_result = after.export_data();

    let result_data = ExecuteAfterResultData {
        before_result,
        after_result,
    };

    // create json_data
    let json_data: String = serde_json::to_string(&result_data).unwrap();

    // execute command
    let exec_commands = create_exec_cmd_args(false, shell_command, after_command);
    let length = exec_commands.len();

    let _ = Command::new(&exec_commands[0])
        .args(&exec_commands[1..length])
        .env("HWATCH_DATA", json_data)
        .spawn();
}

//
fn create_exec_cmd_args(is_exec: bool, shell_command: String, command: String) -> Vec<String> {
    // Declaration at child.
    let mut exec_commands = vec![];
    let mut is_shellcmd_template = false;

    if is_exec {
        // is -x option enable
        exec_commands = shell_words::split(&command).expect("shell command parse error.");
    } else {
        // if `-e` option disable. (default)
        // split self.shell_command
        let shell_commands =
            shell_words::split(&shell_command).expect("shell command parse error.");

        // set shell_command args, to exec_cmd_args.
        exec_commands.push(shell_commands[0].to_string());

        if shell_commands.len() >= 2 {
            // set string command.
            let length = shell_commands.len();
            let shell_command_args = shell_commands[1..length].to_vec();

            // shell_command_args to exec_cmd_args
            for shell_command_arg in shell_command_args {
                let exec_cmd_arg: String;
                if shell_command_arg.contains("{COMMAND}") {
                    exec_cmd_arg =
                        str::replace(&shell_command_arg, crate::SHELL_COMMAND_EXECCMD, &command);
                    is_shellcmd_template = true;
                } else {
                    exec_cmd_arg = shell_command_arg;
                }

                // push exec_cmd_arg to exec_cmd_args
                exec_commands.push(exec_cmd_arg);
            }
        }

        // add exec command..
        if !is_shellcmd_template {
            exec_commands.push(command);
        }
    }

    exec_commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_comparison() {
        let command_result = CommandResult::default();
        let command_result_2 = CommandResult {
            timestamp: "SomeOtherTime".to_string(),
            ..Default::default()
        };
        //Timestamp is not part of the comparison. Let's ensure it's different to prove.
        assert!(command_result == command_result_2);
    }

    #[test]
    fn test_command_result_default() {
        let command_result = CommandResult::default();
        assert_eq!(command_result.command, "".to_string());
    }

    #[test]
    fn test_command_result_clone() {
        let command_result = CommandResult::default();
        let command_result2 = command_result.clone();
        assert!(command_result == command_result2);
    }

    #[test]
    fn test_command_result_equality() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult::default();
        assert!(command_result1 == command_result2);
    }

    #[test]
    fn test_command_result_command_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult {
            command: "different".to_string(),
            ..Default::default()
        };
        assert!(command_result1 != command_result2);
    }

    #[test]
    fn test_command_result_status_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult {
            status: false,
            ..Default::default()
        };
        assert!(command_result1 != command_result2);
    }

    #[test]
    fn test_command_result_output_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult::default().set_output("different".as_bytes().to_vec());
        assert!(command_result1 != command_result2);
    }

    #[test]
    fn test_command_result_stdout_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult::default().set_stdout("different".as_bytes().to_vec());
        assert!(command_result1 != command_result2);
    }

    #[test]
    fn test_command_result_stderr_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult::default().set_stderr("different".as_bytes().to_vec());
        assert!(command_result1 != command_result2);
    }
}
