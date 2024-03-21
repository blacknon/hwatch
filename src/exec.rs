// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use crossbeam_channel::Sender;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

// local module
use crate::common;
use crate::event::AppEvent;

#[derive(Clone, Deserialize, Serialize)]
pub struct CommandResult {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            timestamp: String::default(),
            command: String::default(),
            status: true,
            output: String::default(),
            stdout: String::default(),
            stderr: String::default(),
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

// TODO(blacknon): commandは削除？
pub struct ExecuteCommand {
    pub shell_command: String,
    pub command: Vec<String>,
    pub is_exec: bool,
    pub tx: Sender<AppEvent>,
}

impl ExecuteCommand {
    // set default value
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self {
            shell_command: "".to_string(),
            command: vec![],
            is_exec: false,
            tx,
        }
    }

    // exec command
    // TODO(blacknon): Resultからcommandを削除して、実行時はこのfunctionの引数として受け付けるように改修する？
    pub fn exec_command(&mut self) {
        // set string command.
        let command_str = self.command.clone().join(" ");

        // create exec_commands...
        let exec_commands = create_exec_cmd_args(self.is_exec,self.shell_command.clone(),command_str.clone());

        // exec command...
        let length = exec_commands.len();
        let child_result = Command::new(&exec_commands[0])
            .args(&exec_commands[1..length])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        // merge stdout and stderr
        let mut vec_output = Vec::new();
        let mut vec_stdout = Vec::new();
        let mut vec_stderr = Vec::new();

        // TODO: bufferのdead lockが起きてるので、うまいこと非同期I/Oを使って修正する
        let status = match child_result {
            Ok(mut child) => {
                let child_stdout = child.stdout.take().expect("");
                let child_stderr = child.stderr.take().expect("");

                // let mut stdout = BufReader::new(child_stdout);
                // let mut stderr = BufReader::new(child_stderr);

                // stdout と stderr の出力を格納するためのベクターを準備する
                let arc_vec_output = Arc::new(Mutex::new(Vec::new()));
                let arc_vec_stdout = Arc::new(Mutex::new(Vec::new()));
                let arc_vec_stderr = Arc::new(Mutex::new(Vec::new()));

                // ベクターへの書き込みをスレッド間で共有するために、Arc と Mutex を使用す
                let arc_vec_output_stdout_clone = Arc::clone(&arc_vec_output);
                let arc_vec_output_stderr_clone = Arc::clone(&arc_vec_output);
                let arc_vec_stdout_clone = Arc::clone(&arc_vec_stdout);
                let arc_vec_stderr_clone = Arc::clone(&arc_vec_stderr);

                // stdout を処理するスレッドを開始する
                let stdout_thread = thread::spawn(move || {
                    // BufReader を使用して子プロセスの stdout を読み取る
                    let mut stdout = BufReader::new(child_stdout);
                    let mut buf = Vec::new();
                    // stdout を完全に読み取り、ベクターに書き込む
                    stdout.read_to_end(&mut buf).expect("Failed to read stdout");
                    arc_vec_stdout_clone.lock().unwrap().extend_from_slice(&buf);
                    arc_vec_output_stdout_clone.lock().unwrap().extend_from_slice(&buf);
                });

                // stderr を処理するスレッドを開始する
                let stderr_thread = thread::spawn(move || {
                    // BufReader を使用して子プロセスの stderr を読み取る
                    let mut stderr = BufReader::new(child_stderr);
                    let mut buf = Vec::new();
                    // stderr を完全に読み取り、ベクターに書き込む
                    stderr.read_to_end(&mut buf).expect("Failed to read stderr");
                    arc_vec_stderr_clone.lock().unwrap().extend_from_slice(&buf);
                    arc_vec_output_stderr_clone.lock().unwrap().extend_from_slice(&buf);
                });

                // stdout と stderr のスレッドが終了するまで待機する
                stdout_thread.join().expect("Failed to join stdout thread");
                stderr_thread.join().expect("Failed to join stderr thread");

                // Arc をアンラップし、MutexGuard を取得してベクターを取り出す
                vec_output = Arc::try_unwrap(arc_vec_output).unwrap().into_inner().unwrap();
                vec_stdout = Arc::try_unwrap(arc_vec_stdout).unwrap().into_inner().unwrap();
                vec_stderr = Arc::try_unwrap(arc_vec_stderr).unwrap().into_inner().unwrap();

                // loop {
                //     // stdout
                //     let stdout_bytes = match stdout.fill_buf() {
                //         Ok(stdout) => {
                //             vec_output.write_all(stdout).expect("");
                //             vec_stdout.write_all(stdout).expect("");

                //             stdout.len()
                //         },
                //         other => panic!("Some better error handling here, {other:?}"),
                //     };
                //     stdout.consume(stdout_bytes);

                //     // stderr
                //     let stderr_bytes = match stderr.fill_buf() {
                //         Ok(stderr) => {
                //             vec_output.write_all(stderr).expect("");
                //             vec_stderr.write_all(stderr).expect("");

                //             stderr.len()
                //         },
                //         other => panic!("Some better error handling here, {other:?}"),
                //     };
                //     stderr.consume(stderr_bytes);

                //     if stdout_bytes == 0 && stderr_bytes == 0 {
                //         break;
                //     }
                // }

                // // Memory release.
                // drop(stdout);
                // drop(stderr);

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
            output: String::from_utf8_lossy(&vec_output).to_string(),
            stdout: String::from_utf8_lossy(&vec_stdout).to_string(),
            stderr: String::from_utf8_lossy(&vec_stderr).to_string(),
        };

        // Send result
        let _ = self.tx.send(AppEvent::OutputUpdate(result));
    }
}

// TODO: 変化が発生した時の後処理コマンドを実行するためのstruct
#[derive(Serialize)]
pub struct ExecuteAfterResultData {
    pub before_result: CommandResult,
    pub after_result: CommandResult,
}

pub fn exec_after_command(shell_command: String, after_command: String, before_result: CommandResult, after_result: CommandResult) {
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
                    exec_cmd_arg = str::replace(
                        &shell_command_arg,
                        crate::SHELL_COMMAND_EXECCMD,
                        &command,
                    );
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
        let command_result2 = CommandResult {
            output: "different".to_string(),
            ..Default::default()
        };
        assert!(command_result1 != command_result2);
    }

    #[test]
    fn test_command_result_stdout_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult {
            stdout: "different".to_string(),
            ..Default::default()
        };
        assert!(command_result1 != command_result2);
    }

    #[test]
    fn test_command_result_stderr_diff() {
        let command_result1 = CommandResult::default();
        let command_result2 = CommandResult {
            stderr: "different".to_string(),
            ..Default::default()
        };
        assert!(command_result1 != command_result2);
    }
}
