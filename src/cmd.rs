// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

// local module
use common;
use event::Event;

#[derive(Clone, Deserialize, Serialize)]
pub struct Result {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
}

impl Result {
    pub fn new() -> Self {
        Self {
            timestamp: String::new(),
            command: String::new(),
            status: true,
            output: String::new(),
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

// TODO(blacknon): commandは削除？
pub struct CmdRun {
    pub command: String,
    pub is_exec: bool,
    pub tx: Sender<Event>,
}

impl CmdRun {
    // set default value
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            command: "".to_string(),
            is_exec: false,
            tx: tx,
        }
    }

    // exec command
    // TODO(blacknon): Resultからcommandを削除して、実行時はこのfunctionの引数として受け付けるように改修する？
    pub fn exec_command(&mut self) {
        // generate exec command
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute prog");

        // TODO(blacknon): 何かしらの方法で、シェルの環境変数や関数、エイリアスの継承を行わせてコマンドを実行させる

        // TODO(blacknon):
        // is_execが有効な場合、self.commandを一度parseしてからchildを生成するよう変更する
        //   ex.)
        //     "dig -x hogehoge @8.8.8.8"
        //     => "dig" "-x" "hogehoge" "@8.8.8.8"
        //
        // if self.is_exec {

        // }

        // merge stdout and stderr
        let mut vec_output = Vec::new();
        let mut vec_stdout = Vec::new();
        let mut vec_stderr = Vec::new();
        {
            let stdout = child.stdout.as_mut().expect("");
            let stderr = child.stderr.as_mut().expect("");

            let mut stdout = BufReader::new(stdout);
            let mut stderr = BufReader::new(stderr);

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
        }

        // get command status
        let status = child.wait().expect("");

        // Set result
        let _result = Result {
            timestamp: common::now_str(),
            command: self.command.clone(),
            status: status.success(),
            output: String::from_utf8_lossy(&vec_output).to_string(),
            stdout: String::from_utf8_lossy(&vec_stdout).to_string(),
            stderr: String::from_utf8_lossy(&vec_stderr).to_string(),
        };

        // Send result
        let _ = self.tx.send(Event::OutputUpdate(_result));
    }
}
