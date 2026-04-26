// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): outputやcommandの型をbyteに変更する
// TODO(blacknon): `command`は別のトコで保持するように変更する？(メモリの節約のため)
// TODO(blacknon): HWATCH_DATAのサイズが大きくなりすぎる場合、環境変数に出力できなくなるのでファイルへの出力をするオプションを追加する

// module
use crossbeam_channel::Sender;

// local module
use crate::common;
use crate::event::AppEvent;

#[path = "exec_after_command.rs"]
mod after_command;
#[path = "exec_process.rs"]
mod process;
#[path = "exec_pty.rs"]
mod pty;
#[path = "exec_result.rs"]
mod result;

pub use self::after_command::exec_after_command;
use self::process::{create_exec_cmd_args, exec_command};
pub use self::result::{CommandResult, CommandResultData};

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
        let exec_commands = match create_exec_cmd_args(
            self.is_exec,
            self.shell_command.clone(),
            command_str.clone(),
        ) {
            Ok(exec_commands) => exec_commands,
            Err(err) => {
                let result = CommandResult {
                    timestamp: common::now_str(),
                    command: command_str,
                    status: false,
                    is_compress: self.is_compress,
                    output: vec![],
                    stdout: vec![],
                    stderr: vec![],
                }
                .set_output(format!("{err}\n").into_bytes())
                .set_stderr(format!("{err}\n").into_bytes());

                let _ = self.tx.send(AppEvent::OutputUpdate(result));
                return;
            }
        };

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
    fn test_command_result_data_generate_result_round_trips_compressed_fields() {
        let data = CommandResultData {
            timestamp: "2026-04-08 12:00:00.000".to_string(),
            command: "echo hi".to_string(),
            status: true,
            output: "joined".to_string(),
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
        };

        let result = data.generate_result(true);

        assert!(result.is_compress);
        assert_eq!(result.get_output(), "joined");
        assert_eq!(result.get_stdout(), "stdout");
        assert_eq!(result.get_stderr(), "stderr");
    }

    #[test]
    fn test_command_result_export_data_decodes_compressed_buffers() {
        let result = CommandResult {
            timestamp: "2026-04-08 12:00:00.000".to_string(),
            command: "echo hi".to_string(),
            status: false,
            is_compress: true,
            output: vec![],
            stdout: vec![],
            stderr: vec![],
        }
        .set_output(b"joined".to_vec())
        .set_stdout(b"out".to_vec())
        .set_stderr(b"err".to_vec());

        let exported = result.export_data();

        assert_eq!(exported.output, "joined");
        assert_eq!(exported.stdout, "out");
        assert_eq!(exported.stderr, "err");
        assert!(!exported.status);
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
            create_exec_cmd_args(false, "bash -c {COMMAND}".to_string(), "ls -la".to_string())
                .unwrap();

        assert_eq!(
            exec_commands,
            vec!["bash".to_string(), "-c".to_string(), "ls -la".to_string(),]
        );
    }

    #[test]
    fn test_create_exec_cmd_args_keeps_shell_command_string() {
        let exec_commands =
            create_exec_cmd_args(false, "sh -c".to_string(), "ls -la;pwd".to_string()).unwrap();

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
        )
        .unwrap();

        assert_eq!(
            exec_commands,
            vec![
                "bash".to_string(),
                "-c".to_string(),
                "source ~/.bashrc; ls -la".to_string(),
            ]
        );
    }

    #[test]
    fn test_create_exec_cmd_args_splits_exec_mode_command() {
        let exec_commands =
            create_exec_cmd_args(true, "ignored".to_string(), "echo hello".to_string()).unwrap();

        assert_eq!(exec_commands, vec!["echo".to_string(), "hello".to_string()]);
    }

    #[test]
    fn test_create_exec_cmd_args_rejects_invalid_shell_command() {
        let err =
            create_exec_cmd_args(false, "\"".to_string(), "echo hello".to_string()).unwrap_err();

        assert!(err.contains("shell command parse error"));
    }

    #[test]
    fn test_create_exec_cmd_args_rejects_invalid_exec_command() {
        let err = create_exec_cmd_args(true, "ignored".to_string(), "\"".to_string()).unwrap_err();

        assert!(err.contains("shell command parse error"));
    }
}
