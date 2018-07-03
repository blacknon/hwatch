use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufRead;
use std::process::{Command,Stdio};
use std::sync::mpsc::Sender;

use common;
use event::Event;

#[derive(Clone)]
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
            timestamp: "".to_string(),
            command: "".to_string(),
            status: true,
            output: "".to_string(),
            stdout: "".to_string(),
            stderr: "".to_string(),
        }
    }
}


pub struct CmdRun {
    pub command: String,
    pub interval: u64,
    pub tx: Sender<Event>,
}


impl CmdRun {
    // set default value
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            command: "".to_string(),
            interval: 10,
            tx: tx,
        }
    }

    // exec command
    pub fn exec_command(&mut self) {
        // exec command
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute prog");

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
                        // merge stdout stderr
                        vec_output.write_all(stdout).expect("");
                        vec_output.write_all(stderr).expect("");

                        // stdout
                        vec_stdout.write_all(stdout).expect("");

                        // stderr
                        vec_stderr.write_all(stderr).expect("");

                        (stdout.len(), stderr.len())
                    }
                    other => panic!("Some better error handling here, {:?}", other)
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

        // Send result
        let _result = Result {
            timestamp: common::now_str(),
            command: self.command.clone(),
            status: status.success(),
            output: String::from_utf8_lossy(&vec_output).to_string(),
            stdout: String::from_utf8_lossy(&vec_stdout).to_string(),
            stderr: String::from_utf8_lossy(&vec_stderr).to_string(),
        };
        let _ = self.tx.send(Event::OutputUpdate(_result));

        // history push
        // let history_last_result = History::get_latest_output();
    }
}