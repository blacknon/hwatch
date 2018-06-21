use std::process::Command;

pub struct Cmd {
    pub command: String,
    pub status: bool,
    pub stdout: String,
    pub stderr: String
}

impl Cmd {
    // new
    pub fn new() -> Self {
        Self{
            command: "".to_string(),
            status: false,
            stdout: "".to_string(),
            stderr: "".to_string()
        }
    }

    // run command
    pub fn run(&mut self) {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .output()
            .expect("failed to execute process");

        // set var
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // set self var
        self.status = output.status.success();
        self.stdout = stdout.to_string();
        self.stderr = stderr.to_string();
    }
}