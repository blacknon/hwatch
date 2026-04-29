// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[cfg(unix)]
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
#[cfg(unix)]
use std::os::fd::OwnedFd;
use std::process::{Command, Stdio};
use std::thread;

#[cfg(unix)]
use super::pty::create_raw_pty;

pub(super) fn create_exec_cmd_args(
    is_exec: bool,
    shell_command: String,
    command: String,
) -> Result<Vec<String>, String> {
    let mut exec_commands = vec![];
    let mut is_shellcmd_template = false;

    if is_exec {
        exec_commands = shell_words::split(&command)
            .map_err(|err| format!("shell command parse error: {err}"))?;
    } else {
        let shell_commands = shell_words::split(&shell_command)
            .map_err(|err| format!("shell command parse error: {err}"))?;
        if shell_commands.is_empty() {
            return Err("shell command parse error: shell command is empty".to_string());
        }

        exec_commands.push(shell_commands[0].to_string());

        if shell_commands.len() >= 2 {
            let length = shell_commands.len();
            let shell_command_args = shell_commands[1..length].to_vec();

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

                exec_commands.push(exec_cmd_arg);
            }
        }

        if !is_shellcmd_template {
            exec_commands.push(command);
        }
    }

    Ok(exec_commands)
}

pub(super) fn exec_command(
    exec_commands: &[String],
    is_pty: bool,
) -> (bool, Vec<u8>, Vec<u8>, Vec<u8>) {
    let length = exec_commands.len();
    let mut command = Command::new(&exec_commands[0]);
    command.args(&exec_commands[1..length]);

    #[cfg(unix)]
    let mut stdin_master: Option<OwnedFd> = None;
    #[cfg(not(unix))]
    let stdin_master: Option<()> = None;

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
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        stdout_reader = ReaderHandle::Pipe;
        stderr_reader = ReaderHandle::Pipe;
    }

    let child_result = command.spawn();
    drop(command);
    let _ = stdin_master;
    let mut vec_output = Vec::new();
    let mut vec_stdout = Vec::new();
    let mut vec_stderr = Vec::new();

    let status = match child_result {
        Ok(mut child) => {
            let stdout_thread = match stdout_reader {
                #[cfg(unix)]
                ReaderHandle::Fd(fd) => thread::spawn(move || read_from_fd(fd, "stdout")),
                ReaderHandle::Pipe => match child.stdout.take() {
                    Some(child_stdout) => {
                        thread::spawn(move || read_from_pipe(child_stdout, "stdout"))
                    }
                    None => thread::spawn(|| {
                        Err("stdout pipe was not available on spawned process".to_string())
                    }),
                },
            };
            let stderr_thread = match stderr_reader {
                #[cfg(unix)]
                ReaderHandle::Fd(fd) => thread::spawn(move || read_from_fd(fd, "stderr")),
                ReaderHandle::Pipe => match child.stderr.take() {
                    Some(child_stderr) => {
                        thread::spawn(move || read_from_pipe(child_stderr, "stderr"))
                    }
                    None => thread::spawn(|| {
                        Err("stderr pipe was not available on spawned process".to_string())
                    }),
                },
            };

            let status = if is_pty {
                child.wait().map(|status| status.success()).unwrap_or(false)
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
                child.wait().map(|status| status.success()).unwrap_or(false)
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
fn read_from_fd(fd: OwnedFd, label: &str) -> Result<Vec<u8>, String> {
    use std::io::ErrorKind;

    let mut file = File::from(fd);
    let mut buf = Vec::new();
    let mut chunk = [0_u8; 8192];

    loop {
        match file.read(&mut chunk) {
            Ok(0) => break,
            Ok(size) => buf.extend_from_slice(&chunk[..size]),
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) if err.kind() == ErrorKind::UnexpectedEof || err.raw_os_error() == Some(5) => {
                break;
            }
            Err(err) => return Err(format!("Failed to read {label}: {err}")),
        }
    }

    Ok(buf)
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
