// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): outputやcommandの型をbyteに変更する
// TODO(blacknon): `command`は別のトコで保持するように変更する？(メモリの節約のため)

// module
use chardetng::EncodingDetector;
use crossbeam_channel::Sender;
use flate2::{read::GzDecoder, write::GzEncoder};
#[cfg(unix)]
use nix::pty::{openpty, OpenptyResult, Winsize};
#[cfg(unix)]
use nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
#[cfg(unix)]
use std::os::fd::OwnedFd;
use std::process::{Command, Stdio};
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
            let mut decoded = Vec::new();
            let _ = decoder.read_to_end(&mut decoded);
            decode_bytes(&decoded)
        } else {
            decode_bytes(data)
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

fn decode_bytes(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut detector = EncodingDetector::new();
    detector.feed(data, true);
    let encoding = detector.guess(None, true);
    let (cow, _, _) = encoding.decode(data);
    cow.into_owned()
}

// TODO(blacknon): commandは削除？
pub struct ExecuteCommand {
    pub shell_command: String,
    pub command: Vec<String>,
    pub is_exec: bool,
    pub is_compress: bool,
    pub is_pty: bool,
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
            is_pty: false,
            tx,
        }
    }

    // exec command
    // TODO(blacknon): Resultからcommandを削除して、実行時はこのfunctionの引数として受け付けるように改修する？
    pub fn exec_command(&mut self) {
        let command_str = self.command.clone().join(" ");

        // create exec_commands...
        let exec_commands = create_exec_cmd_args(
            self.is_exec,
            self.shell_command.clone(),
            command_str.clone(),
        );

        let (status, vec_output, vec_stdout, vec_stderr) =
            exec_command(&exec_commands, self.is_pty);

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
                if shell_command_arg == crate::SHELL_COMMAND_EXECCMD
                    && should_append_command_to_previous_arg(&exec_commands)
                {
                    if let Some(previous_arg) = exec_commands.last_mut() {
                        previous_arg.push(' ');
                        previous_arg.push_str(&command);
                        is_shellcmd_template = true;
                        continue;
                    }
                }

                let exec_cmd_arg = if shell_command_arg.contains("{COMMAND}") {
                    is_shellcmd_template = true;
                    str::replace(&shell_command_arg, crate::SHELL_COMMAND_EXECCMD, &command)
                } else {
                    shell_command_arg
                };

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

fn exec_command(exec_commands: &[String], is_pty: bool) -> (bool, Vec<u8>, Vec<u8>, Vec<u8>) {
    let length = exec_commands.len();
    let mut command = Command::new(&exec_commands[0]);
    command.args(&exec_commands[1..length]);

    let mut stdin_master = None;
    let stdout_reader;
    let stderr_reader;

    #[cfg(unix)]
    {
        if is_pty {
            let stdin_pty = create_raw_pty();
            let stdout_pty = create_raw_pty();
            let stderr_pty = create_raw_pty();

            let (stdin_pty, stdout_pty, stderr_pty) = match (stdin_pty, stdout_pty, stderr_pty) {
                (Ok(stdin_pty), Ok(stdout_pty), Ok(stderr_pty)) => {
                    (stdin_pty, stdout_pty, stderr_pty)
                }
                (Err(err), _, _) | (_, Err(err), _) | (_, _, Err(err)) => {
                    let error_msg = err.to_string().into_bytes();
                    return (false, Vec::new(), Vec::new(), error_msg);
                }
            };

            stdin_master = Some(stdin_pty.master);
            stdout_reader = ReaderHandle::Fd(stdout_pty.master);
            stderr_reader = ReaderHandle::Fd(stderr_pty.master);

            command
                .stdin(Stdio::from(stdin_pty.slave))
                .stdout(Stdio::from(stdout_pty.slave))
                .stderr(Stdio::from(stderr_pty.slave));
        } else {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
            stdout_reader = ReaderHandle::Pipe;
            stderr_reader = ReaderHandle::Pipe;
        }
    }

    #[cfg(not(unix))]
    {
        // On non-unix targets PTY support isn't available; always use pipes.
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        stdout_reader = ReaderHandle::Pipe;
        stderr_reader = ReaderHandle::Pipe;
    }

    let child_result = command.spawn();
    drop(command);
    drop(stdin_master);
    let mut vec_output = Vec::new();
    let mut vec_stdout = Vec::new();
    let mut vec_stderr = Vec::new();

    let status = match child_result {
        Ok(mut child) => {
            let stdout_thread = match stdout_reader {
                ReaderHandle::Fd(fd) => thread::spawn(move || read_from_fd(fd, "stdout")),
                ReaderHandle::Pipe => {
                    let child_stdout = child.stdout.take().expect("");
                    thread::spawn(move || read_from_pipe(child_stdout, "stdout"))
                }
            };
            let stderr_thread = match stderr_reader {
                ReaderHandle::Fd(fd) => thread::spawn(move || read_from_fd(fd, "stderr")),
                ReaderHandle::Pipe => {
                    let child_stderr = child.stderr.take().expect("");
                    thread::spawn(move || read_from_pipe(child_stderr, "stderr"))
                }
            };

            let status = if is_pty {
                child.wait().expect("").success()
            } else {
                false
            };
            vec_stdout = stdout_thread
                .join()
                .unwrap_or_else(|_| Err("Failed to join stdout thread".to_string()))
                .unwrap_or_else(|err| format!("{err}\n").into_bytes());
            vec_stderr = stderr_thread
                .join()
                .unwrap_or_else(|_| Err("Failed to join stderr thread".to_string()))
                .unwrap_or_else(|err| format!("{err}\n").into_bytes());
            vec_output = vec_stdout.clone();
            vec_output.extend_from_slice(&vec_stderr);

            if is_pty {
                status
            } else {
                child.wait().expect("").success()
            }
        }
        Err(err) => {
            let error_msg = err.to_string();

            let mut stdout_text: Vec<u8> = error_msg.as_bytes().to_vec();
            let mut stderr_text: Vec<u8> = error_msg.as_bytes().to_vec();
            vec_output.append(&mut stdout_text);
            vec_stderr.append(&mut stderr_text);

            false
        }
    };

    (status, vec_output, vec_stdout, vec_stderr)
}

#[cfg(unix)]
enum ReaderHandle {
    Pipe,
    Fd(OwnedFd),
}

#[cfg(not(unix))]
enum ReaderHandle {
    Pipe,
}

#[cfg(unix)]
fn create_raw_pty() -> Result<OpenptyResult, nix::Error> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let result = openpty(Some(&winsize), None)?;

    let mut termios = tcgetattr(&result.slave)?;
    cfmakeraw(&mut termios);
    tcsetattr(&result.slave, SetArg::TCSANOW, &termios)?;

    Ok(result)
}

#[cfg(unix)]
fn read_from_fd(fd: OwnedFd, label: &str) -> Result<Vec<u8>, String> {
    let file = File::from(fd);
    read_from_pipe(file, label)
}

fn read_from_pipe<R: Read>(reader: R, label: &str) -> Result<Vec<u8>, String> {
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|err| format!("Failed to read {label}: {err}"))?;
    Ok(buf)
}

fn should_append_command_to_previous_arg(exec_commands: &[String]) -> bool {
    exec_commands.len() >= 3 && exec_commands[1] == "-c"
}

#[cfg(test)]
mod tests {
    use super::*;
    use encoding_rs::SHIFT_JIS;

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

    #[test]
    fn test_decode_shift_jis_bytes() {
        let (encoded, _, _) = SHIFT_JIS.encode("日本語テスト");
        let command_result = CommandResult::default().set_output(encoded.into_owned());
        assert_eq!(command_result.get_output(), "日本語テスト");
    }

    #[test]
    fn test_exec_command_without_force_color_stdout_is_not_tty() {
        let exec_commands = vec![
            "sh".to_string(),
            "-c".to_string(),
            "if [ -t 1 ]; then printf tty; else printf notty; fi".to_string(),
        ];

        let (_, _, stdout, _) = exec_command(&exec_commands, false);
        assert_eq!(String::from_utf8(stdout).unwrap(), "notty");
    }

    #[test]
    fn test_exec_command_with_force_color_stdout_is_tty() {
        let exec_commands = vec![
            "sh".to_string(),
            "-c".to_string(),
            "if [ -t 1 ]; then printf tty; else printf notty; fi".to_string(),
        ];

        let (_, _, stdout, _) = exec_command(&exec_commands, true);
        assert_eq!(String::from_utf8(stdout).unwrap(), "tty");
    }

    #[test]
    fn test_exec_command_with_force_color_stdin_is_tty() {
        let exec_commands = vec![
            "sh".to_string(),
            "-c".to_string(),
            "if [ -t 0 ]; then printf tty; else printf notty; fi".to_string(),
        ];

        let (_, _, stdout, _) = exec_command(&exec_commands, true);
        assert_eq!(String::from_utf8(stdout).unwrap(), "tty");
    }

    #[test]
    fn test_create_exec_cmd_args_replaces_template_in_argument() {
        let exec_commands =
            create_exec_cmd_args(false, "bash -c {COMMAND}".to_string(), "ls -la".to_string());

        assert_eq!(
            exec_commands,
            vec!["bash".to_string(), "-c".to_string(), "ls -la".to_string(),]
        );
    }

    #[test]
    fn test_create_exec_cmd_args_keeps_shell_command_string() {
        let exec_commands =
            create_exec_cmd_args(false, "sh -c".to_string(), "ls -la;pwd".to_string());

        assert_eq!(
            exec_commands,
            vec!["sh".to_string(), "-c".to_string(), "ls -la;pwd".to_string(),]
        );
    }

    #[test]
    fn test_create_exec_cmd_args_appends_command_after_dash_c_script() {
        let exec_commands = create_exec_cmd_args(
            false,
            "bash -c \"source ~/.bashrc\"; {COMMAND}".to_string(),
            "ls -la".to_string(),
        );

        assert_eq!(
            exec_commands,
            vec![
                "bash".to_string(),
                "-c".to_string(),
                "source ~/.bashrc; ls -la".to_string(),
            ]
        );
    }
}
