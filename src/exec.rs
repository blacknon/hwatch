// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use crossbeam_channel::Sender;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Child, Command, Stdio};

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
            tx: tx,
        }
    }

    // exec command
    // TODO(blacknon): Resultからcommandを削除して、実行時はこのfunctionの引数として受け付けるように改修する？
    pub fn exec_command(&mut self) {
        // Declaration at child.
        let exec_cmd: String;
        let mut exec_cmd_args = vec![];
        let mut is_shellcmd_template = false;

        // set string command.
        let command_str = self.command.clone().join(" ");

        if self.is_exec {
            // is -x option enable
            // command parse
            let length = self.command.len();
            exec_cmd = self.command[0].clone();
            exec_cmd_args = self.command[1..length].to_vec();
        } else {
            // if `-e` option disable. (default)
            // split self.shell_command
            let shell_commands =
                shell_words::split(&self.shell_command).expect("shell command parse error.");

            // set shell_command args, to exec_cmd_args.
            exec_cmd = shell_commands[0].to_string();
            if shell_commands.len() >= 2 {
                let length = shell_commands.len();
                let shell_command_args = shell_commands[1..length].to_vec();

                // shell_command_args to exec_cmd_args
                for shell_command_arg in shell_command_args {
                    let exec_cmd_arg: String;
                    if shell_command_arg.contains("{COMMAND}") {
                        exec_cmd_arg = str::replace(
                            &shell_command_arg,
                            crate::SHELL_COMMAND_EXECCMD,
                            &command_str,
                        );
                        is_shellcmd_template = true;
                    } else {
                        exec_cmd_arg = shell_command_arg;
                    }

                    // push exec_cmd_arg to exec_cmd_args
                    exec_cmd_args.push(exec_cmd_arg);
                }
            }

            // add exec command..
            if !is_shellcmd_template {
                exec_cmd_args.push(command_str.clone());
            }
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
            let err = child_result.unwrap_err();
            let error_msg = err.to_string();

            let mut stdout_text: Vec<u8> = error_msg.as_bytes().to_vec();
            let mut stderr_text: Vec<u8> = error_msg.as_bytes().to_vec();
            vec_output.append(&mut stdout_text);
            vec_stderr.append(&mut stderr_text);

            // get command status
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
