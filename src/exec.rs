// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use crossbeam_channel::Sender;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Child, Command, ExitStatus, Stdio};

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

// TODO(blacknon): commandは削除？
pub struct ExecuteCommand {
    pub command: Vec<String>,
    pub is_exec: bool,
    pub tx: Sender<AppEvent>,
}

struct CommandOutput {
    pub status: bool,
    pub vec_output: Vec<u8>,
    pub vec_stdout: Vec<u8>,
    pub vec_stderr: Vec<u8>,
}

impl ExecuteCommand {
    // set default value
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self {
            command: vec![],
            is_exec: false,
            tx: tx,
        }
    }

    // exec command
    // TODO(blacknon): Resultからcommandを削除して、実行時はこのfunctionの引数として受け付けるように改修する？
    // TODO(blacknon): Windowsに対応していないのでどうにかする
    pub fn exec_command(&mut self) {
        // Declaration at child.
        let exec_cmd: String;
        let mut exec_cmd_args: Vec<String>;

        // set string command.
        let command_str = shell_words::join(self.command.clone());

        // if `-e` option enable.
        if !self.is_exec {
            if cfg!(windows) {
                exec_cmd = "cmd".to_string();
                exec_cmd_args = vec!["/C".to_string()];
            } else {
                exec_cmd = "sh".to_string();
                exec_cmd_args = vec!["-c".to_string()];
            }

            // add exec command..
            exec_cmd_args.push(command_str.clone());
        } else {
            // command parse
            let length = self.command.len();
            exec_cmd = self.command[0].clone();
            exec_cmd_args = self.command[1..length].to_vec();
        }

        // exec command...
        let child_result = Command::new(exec_cmd)
            .args(exec_cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child: Child;
        let status: bool;

        // merge stdout and stderr
        let mut vec_output = Vec::new();
        let mut vec_stdout = Vec::new();
        let mut vec_stderr = Vec::new();

        if child_result.is_ok() {
            child = child_result.expect("failed to execute process.");

            let child_stdout = child.stdout.as_mut().expect("");
            let child_stderr = child.stderr.as_mut().expect("");

            let mut stdout = BufReader::new(child_stdout);
            let mut stderr = BufReader::new(child_stderr);

            loop {
                let (stdout_bytes, stderr_bytes) = match (stdout.fill_buf(), stderr.fill_buf()) {
                    (Ok(stdout), Ok(stderr)) => {
                        // merge stdout/stderr
                        vec_output.write_all(stdout).expect("");
                        vec_output.write_all(stderr).expect("");

                        // stdout
                        vec_stdout.write_all(stdout).expect("");

                        // stderr
                        vec_stderr.write_all(stderr).expect("");

                        (stdout.len(), stderr.len())
                    }
                    other => panic!("Some better error handling here, {:?}", other),
                };

                if stdout_bytes == 0 && stderr_bytes == 0 {
                    break;
                }

                stdout.consume(stdout_bytes);
                stderr.consume(stderr_bytes);
            }

            // Memory release.
            drop(stdout);
            drop(stderr);

            // get command status
            let exit_status = child.wait().expect("");
            status = exit_status.success();
        } else {
            let mut stdout_text: Vec<u8> = "failed to execute process.".as_bytes().to_vec();
            let mut stderr_text: Vec<u8> = "failed to execute process.".as_bytes().to_vec();
            vec_output.append(&mut stdout_text);
            vec_stderr.append(&mut stderr_text);

            status = false;
        }

        // Set result
        let result = CommandResult {
            timestamp: common::now_str(),
            command: command_str,
            status: status,
            output: String::from_utf8_lossy(&vec_output).to_string(),
            stdout: String::from_utf8_lossy(&vec_stdout).to_string(),
            stderr: String::from_utf8_lossy(&vec_stderr).to_string(),
        };

        // Send result
        let _ = self.tx.send(AppEvent::OutputUpdate(result));
    }
}
