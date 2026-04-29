// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use clap::{
    builder::ArgPredicate, crate_authors, crate_description, crate_name, crate_version, Arg,
    ArgAction, Command, ValueHint,
};
use std::collections::HashSet;
use std::env::args;
use std::ffi::OsString;

use crate::{common, HISTORY_LIMIT, SHELL_COMMAND};

pub fn build_app() -> Command {
    let _program = args()
        .next()
        .and_then(|s| {
            std::path::PathBuf::from(s)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap();

    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("command")
                .action(ArgAction::Append)
                .num_args(1..)
                .value_hint(ValueHint::CommandWithArguments)
                .trailing_var_arg(true),
        )
        .arg(
            Arg::new("batch")
                .help("output execution results to stdout")
                .short('b')
                .action(ArgAction::SetTrue)
                .long("batch"),
        )
        .arg(
            Arg::new("beep")
                .help("beep if command has a change result")
                .short('B')
                .action(ArgAction::SetTrue)
                .long("beep"),
        )
        .arg(
            Arg::new("chgexit")
                .help("exit when output changes. With no value, exits after the first change; with N, exits after N changes")
                .short('g')
                .long("chgexit")
                .num_args(0..=1)
                .default_missing_value("1")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("border")
                .help("Surround each pane with a border frame")
                .action(ArgAction::SetTrue)
                .long("border"),
        )
        .arg(
            Arg::new("with_scrollbar")
                .help("When the border option is enabled, display scrollbar on the right side of watch pane.")
                .action(ArgAction::SetTrue)
                .long("with-scrollbar"),
        )
        .arg(
            Arg::new("mouse")
                .help("enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.")
                .action(ArgAction::SetTrue)
                .long("mouse"),
        )
        .arg(
            Arg::new("color")
                .help("interpret ANSI color and style sequences")
                .short('c')
                .action(ArgAction::SetTrue)
                .long("color"),
        )
        .arg(
            Arg::new("reverse")
                .help("display text upside down.")
                .short('r')
                .action(ArgAction::SetTrue)
                .long("reverse"),
        )
        .arg(
            Arg::new("compress")
                .help("Compress data in memory. Note: If the output of the command is small, you may not get the desired effect.")
                .short('C')
                .action(ArgAction::SetTrue)
                .long("compress"),
        )
        .arg(
            Arg::new("no_title")
                .help("hide the UI on start. Use `t` to toggle it.")
                .long("no-title")
                .action(ArgAction::SetTrue)
                .short('t'),
        )
        .arg(
            Arg::new("enable_summary_char")
                .help("collect character-level diff count in summary.")
                .long("enable-summary-char")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("line_number")
                .help("show line number")
                .short('N')
                .action(ArgAction::SetTrue)
                .long("line-number"),
        )
        .arg(
            Arg::new("wrap")
                .help("disable line wrap mode")
                .short('w')
                .action(ArgAction::SetTrue)
                .long("wrap"),
        )
        .arg(
            Arg::new("no_help_banner")
                .help("hide the \"Display help with h key\" message")
                .long("no-help-banner")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_summary")
                .help("disable the calculation for summary that is running behind the scenes, and disable the summary function in the first place.")
                .long("no-summary")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("completion")
                .help("Output shell completion script")
                .long("completion")
                .value_name("SHELL")
                .value_parser(["bash", "fish", "zsh"])
                .action(ArgAction::Set)
                .exclusive(true),
        )
        .arg(
            Arg::new("exec")
                .help("Run the command directly, not through the shell. Much like the `-x` option of the watch command.")
                .short('x')
                .action(ArgAction::SetTrue)
                .long("exec"),
        )
        .arg(
            Arg::new("use_pty")
                .help("Run the command through a pseudo-TTY so commands that colorize on terminals can keep color output.")
                .long("use-pty")
                .short('p')
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("diff_output_only")
                .help("Display only the lines with differences during `line` diff and `word` diff.")
                .short('O')
                .long("diff-output-only")
                .requires("differences")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ignore_spaceblock")
                .help("Ignore diffs where only consecutive whitespace blocks differ.")
                .long("ignore-spaceblock")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("after_command")
                .help("Executes the specified command if the output changes. Information about changes is stored in json format in environment variable ${HWATCH_DATA}.")
                .short('A')
                .long("aftercommand")
                .value_hint(ValueHint::CommandString)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("after_command_result_write_file")
                .help("Passes `${HWATCH_DATA}` to `aftercommand` as a temporary file path instead of inline json data.")
                .long("after-command-result-write-file")
                .requires("after_command")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("logfile")
                .help("logging file. if a log file is already used, its contents will be read and executed.")
                .short('l')
                .long("logfile")
                .num_args(0..=1)
                .value_hint(ValueHint::FilePath)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("force_logfile_overwrite")
                .help("continue even if an existing logfile is empty or unreadable")
                .long("force-logfile-overwrite")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("shell_command")
                .help("shell to use at runtime. can also insert the command to the location specified by {COMMAND}.")
                .short('s')
                .long("shell")
                .action(ArgAction::Append)
                .value_hint(ValueHint::CommandString)
                .default_value(SHELL_COMMAND),
        )
        .arg(
            Arg::new("interval")
                .help("seconds to wait between updates")
                .short('n')
                .long("interval")
                .num_args(1)
                .value_parser(clap::value_parser!(f64))
                .default_value("2"),
        )
        .arg(
            Arg::new("precise")
                .help("Attempt to run as close to the interval as possible, regardless of how long the command takes to run")
                .long("precise")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("limit")
                .help("Set the number of history records to keep. only work in watch mode. Set `0` for unlimited recording.")
                .short('L')
                .long("limit")
                .value_parser(clap::value_parser!(u32))
                .default_value(HISTORY_LIMIT),
        )
        .arg(
            Arg::new("tab_size")
                .help("Specifying tab display size")
                .long("tab-size")
                .value_parser(clap::value_parser!(u16))
                .action(ArgAction::Append)
                .default_value("4"),
        )
        .arg(
            Arg::new("diff_plugin")
                .help("Load a diffmode plugin dynamic library.")
                .long("diff-plugin")
                .value_hint(ValueHint::FilePath)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("differences")
                .help("highlight changes between updates")
                .long("differences")
                .short('d')
                .num_args(0..=1)
                .default_missing_value("watch")
                .default_value_ifs([("differences", ArgPredicate::IsPresent, None)])
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("output")
                .help("Select command output.")
                .short('o')
                .long("output")
                .num_args(0..=1)
                .value_parser(["output", "stdout", "stderr"])
                .default_value("output")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("keymap")
                .help("Add keymap")
                .short('K')
                .long("keymap")
                .action(ArgAction::Append),
        )
}

pub fn get_clap_matcher(cmd_app: Command) -> clap::ArgMatches {
    let mut os_args = std::env::args_os();
    let program = os_args
        .next()
        .unwrap_or_else(|| OsString::from("args-join-sample"));

    let env_config = std::env::var("HWATCH").unwrap_or_default();
    let env_tokens: Vec<String> = if env_config.is_empty() {
        Vec::new()
    } else {
        match shell_words::split(&env_config) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warning: failed to parse $HWATCH: {}", e);
                Vec::new()
            }
        }
    };

    let cli_tokens: Vec<String> = os_args.map(|s| s.to_string_lossy().into_owned()).collect();

    let mut joined: Vec<OsString> = Vec::with_capacity(1 + env_tokens.len() + cli_tokens.len());
    joined.push(program);
    for t in env_tokens {
        joined.push(OsString::from(t));
    }
    for t in cli_tokens {
        joined.push(OsString::from(t));
    }

    let joined = normalize_args(joined);

    cmd_app.get_matches_from(joined)
}

pub fn should_continue_with_unreadable_logfile(
    force_logfile_overwrite: bool,
    stdin_is_terminal: bool,
) -> bool {
    if force_logfile_overwrite {
        return true;
    }

    if !stdin_is_terminal {
        eprintln!(
            "Refusing to reuse the existing logfile without confirmation in a non-interactive session. Rerun with --force-logfile-overwrite to continue."
        );
        return false;
    }

    common::confirm_yes_default("Log to the same file?")
}

fn builtin_diff_mode_names() -> HashSet<String> {
    HashSet::from([
        "none".to_string(),
        "watch".to_string(),
        "line".to_string(),
        "word".to_string(),
    ])
}

fn has_diff_plugin_option(args: &[OsString]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_string_lossy();
        if arg == "--" {
            break;
        }
        if arg == "--diff-plugin" {
            return true;
        }
        if arg.starts_with("--diff-plugin=") {
            return true;
        }
        index += 1;
    }

    false
}

fn normalize_args(args: Vec<OsString>) -> Vec<OsString> {
    let known_diff_modes = builtin_diff_mode_names();
    let has_diff_plugin = has_diff_plugin_option(&args);
    let mut normalized = Vec::with_capacity(args.len() + 1);
    let mut iter = args.into_iter();

    if let Some(program) = iter.next() {
        normalized.push(program);
    }

    while let Some(arg) = iter.next() {
        if arg == "--" {
            normalized.push(arg);
            normalized.extend(iter);
            break;
        }

        if arg == "-d" || arg == "--differences" {
            normalized.push(arg);

            match iter.next() {
                Some(next) => {
                    let next_value = next.to_string_lossy();
                    if next_value.starts_with('-') {
                        normalized.push(OsString::from("watch"));
                        normalized.push(next);
                    } else if known_diff_modes.contains(next_value.as_ref()) || has_diff_plugin {
                        normalized.push(next);
                    } else {
                        normalized.push(OsString::from("watch"));
                        normalized.push(next);
                    }
                }
                None => normalized.push(OsString::from("watch")),
            }

            continue;
        }

        if arg == "-g" || arg == "--chgexit" {
            normalized.push(arg);

            match iter.next() {
                Some(next) => {
                    let next_value = next.to_string_lossy();
                    if next_value.parse::<u32>().is_ok() {
                        normalized.push(next);
                    } else {
                        normalized.push(OsString::from("1"));
                        normalized.push(next);
                    }
                }
                None => normalized.push(OsString::from("1")),
            }

            continue;
        }

        normalized.push(arg);
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{build_app, normalize_args, should_continue_with_unreadable_logfile};
    use std::ffi::OsString;

    #[test]
    fn normalize_args_keeps_explicit_mode() {
        let args = vec!["hwatch", "-d", "line", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();

        let actual: Vec<String> = normalize_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        assert_eq!(actual, vec!["hwatch", "-d", "line", "echo", "hi"]);
    }

    #[test]
    fn normalize_args_inserts_default_mode_before_command() {
        let args = vec!["hwatch", "-d", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();

        let actual: Vec<String> = normalize_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        assert_eq!(actual, vec!["hwatch", "-d", "watch", "echo", "hi"]);
    }

    #[test]
    fn normalize_args_inserts_default_mode_before_next_option() {
        let args = vec!["hwatch", "--differences", "--batch", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();

        let actual: Vec<String> = normalize_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            actual,
            vec!["hwatch", "--differences", "watch", "--batch", "echo", "hi"]
        );
    }

    #[test]
    fn normalize_args_preserves_non_builtin_mode_when_plugin_option_is_present() {
        let args = vec![
            "hwatch",
            "--diff-plugin",
            "/tmp/libnumeric.so",
            "-d",
            "numeric-diff",
            "echo",
            "hi",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();

        let actual: Vec<String> = normalize_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            actual,
            vec![
                "hwatch",
                "--diff-plugin",
                "/tmp/libnumeric.so",
                "-d",
                "numeric-diff",
                "echo",
                "hi",
            ]
        );
    }

    #[test]
    fn normalize_args_still_defaults_to_watch_without_plugin_option() {
        let args = vec!["hwatch", "-d", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();

        let actual: Vec<String> = normalize_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        assert_eq!(actual, vec!["hwatch", "-d", "watch", "echo", "hi"]);
    }

    #[test]
    fn clap_parses_chgexit_without_value_as_one() {
        let args = vec!["hwatch", "-g", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();
        let matches = build_app()
            .try_get_matches_from(normalize_args(args))
            .unwrap();

        assert_eq!(matches.get_one::<u32>("chgexit"), Some(&1));
    }

    #[test]
    fn clap_parses_chgexit_with_explicit_count() {
        let args = vec!["hwatch", "-g", "3", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();
        let matches = build_app()
            .try_get_matches_from(normalize_args(args))
            .unwrap();

        assert_eq!(matches.get_one::<u32>("chgexit"), Some(&3));
    }

    #[test]
    fn clap_parses_force_logfile_overwrite_flag() {
        let args = vec!["hwatch", "--force-logfile-overwrite", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();
        let matches = build_app()
            .try_get_matches_from(normalize_args(args))
            .unwrap();

        assert!(matches.get_flag("force_logfile_overwrite"));
    }

    #[test]
    fn normalize_args_inserts_default_count_for_chgexit() {
        let args = vec!["hwatch", "-g", "echo", "hi"]
            .into_iter()
            .map(OsString::from)
            .collect();

        let actual: Vec<String> = normalize_args(args)
            .into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        assert_eq!(actual, vec!["hwatch", "-g", "1", "echo", "hi"]);
    }

    #[test]
    fn unreadable_logfile_requires_force_in_non_interactive_mode() {
        assert!(!should_continue_with_unreadable_logfile(false, false));
        assert!(should_continue_with_unreadable_logfile(true, false));
    }
}
