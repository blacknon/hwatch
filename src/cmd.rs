use std::process::Command;
use std::sync::mpsc::Sender;

use common;
use event::Event;
use event::Event::OutputUpdate;

pub struct Cmd {
    pub timestamp: String,
    pub command: String,
    pub status: bool,
    pub stdout: String,
    pub stderr: String,
}

pub struct CmdRun {
    pub cmd: Cmd,
    pub tx: Sender<Event>

}

impl CmdRun {
    // set default value
    pub fn new(tx: Sender<Event>) -> Self {
        let cmd = Cmd {
            timestamp: "".to_string(),
            command: "".to_string(),
            status: false,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };

        Self {
            cmd: cmd,
            tx: tx
        }
    }

    // run command
    pub fn exec(&mut self) {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd.command)
            .output()
            .expect("failed to execute process");

        // set var
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // set self var
        self.cmd.status = output.status.success();
        self.cmd.stdout = stdout.to_string();
        self.cmd.stderr = stderr.to_string();

        let _cmd = Cmd {
            timestamp: common::now_str(),
            command: self.cmd.command.clone(),
            status: self.cmd.status,
            stdout: self.cmd.stdout.clone(),
            stderr: self.cmd.stderr.clone()
        };

        let _ =self.tx.send(OutputUpdate(_cmd));
    }
}