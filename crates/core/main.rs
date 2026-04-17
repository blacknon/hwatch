// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.4.0
// TODO(blacknon): 空白の数だけ違う場合、diffとして扱わないようにするオプションの追加(shortcut keyではなく、`:set hogehoge...`で指定する機能として実装)
// TODO(blacknon): diff modeをさらに複数用意し、選択・切り替えできるdiffをオプションから指定できるようにする(watchをold-watchにして、モダンなwatchをデフォルトにしたり)
// TODO(blacknon): formatを整える機能や、diff時に特定のフォーマットかどうかで扱いを変える機能について、追加する方法を模索する(プラグインか、もしくはパイプでうまいこときれいにする機能か？)q
// TODO(blacknon): filter modeのハイライト表示の色を環境変数で定義できるようにする
// TODO(blacknon): filter modeの検索ヒット数を表示する(どうやってやろう…？というより、どこに表示させよう…？)
// TODO(blacknon): Windowsのバイナリをパッケージマネジメントシステムでインストール可能になるよう、Releaseでうまいこと処理をする
// TODO(blacknon): watchをモダンよりのものに変更する

// v1.0.0
// TODO(blacknon): vimのように内部コマンドを利用した表示切り替え・出力結果の編集機能を追加する
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能を追加する(command mode option)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man
// TODO(blacknon): エラーなどのメッセージ表示領域の作成

// crate
extern crate ansi_parser;
extern crate ansi_term;
extern crate async_std;
extern crate chardetng;
extern crate chrono;
extern crate config;
extern crate crossbeam_channel;
extern crate crossterm;
extern crate ctrlc;
extern crate encoding_rs;
extern crate flate2;
extern crate futures;
extern crate heapless;
extern crate nix;
extern crate question;
extern crate ratatui as tui;
extern crate regex;
extern crate serde;
extern crate shell_words;
extern crate similar;
extern crate tempfile;
extern crate termwiz;
extern crate tokio;
extern crate unicode_segmentation;
extern crate unicode_width;

// local crate
extern crate hwatch_ansi;
extern crate hwatch_diffmode;

#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
extern crate termios;

// macro crate
#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

// modules
use clap::{builder::ArgPredicate, error::ErrorKind, Arg, ArgAction, Command, ValueHint};
use common::load_logfile;
use crossbeam_channel::unbounded;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use hwatch_diffmode::DiffMode;
use question::{Answer, Question};
use std::collections::{HashMap, HashSet};
use std::env::args;
use std::ffi::OsString;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};
use unicode_width::UnicodeWidthStr;

// local modules
mod app;
mod batch;
mod common;
mod diffmode_line;
mod diffmode_plane;
mod diffmode_watch;
mod errors;
mod event;
mod exec;
mod header;
mod help;
mod history;
mod keymap;
mod output;
mod plugin_diffmode;
mod popup;
mod view;
mod watch;

// const
pub const DEFAULT_TAB_SIZE: u16 = 4;
pub const HISTORY_WIDTH: u16 = 25;
pub const SHELL_COMMAND_EXECCMD: &str = "{COMMAND}";
pub const HISTORY_LIMIT: &str = "5000";
type SharedInterval = Arc<RwLock<RunInterval>>;

#[derive(Clone, Debug)]
struct RunInterval {
    interval: f64,
    paused: bool,
}

impl RunInterval {
    fn new(interval: f64) -> Self {
        Self {
            interval,
            paused: false,
        }
    }
    fn increase(&mut self, seconds: f64) {
        self.interval += seconds;
    }
    fn decrease(&mut self, seconds: f64) {
        if self.interval > seconds {
            self.interval -= seconds;
        }
    }
    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}
impl Default for RunInterval {
    fn default() -> Self {
        Self::new(2.0)
    }
}

// const at Windows
#[cfg(windows)]
const SHELL_COMMAND: &str = "cmd /C";

// const at not Windows
#[cfg(not(windows))]
const SHELL_COMMAND: &str = "sh -c";

/// Parse args and options function.
fn build_app() -> clap::Command {
    // get own name
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
        // .trailing_var_arg(true)
        .author(crate_authors!())

        // -- command --
        .arg(
            Arg::new("command")
                .action(ArgAction::Append)
                // .allow_hyphen_values(true)
                .num_args(1..)
                .value_hint(ValueHint::CommandWithArguments)
                .trailing_var_arg(true)
        )

        // -- flags --
        // Enable batch mode option
        //     [-b,--batch]
        .arg(
            Arg::new("batch")
                .help("output execution results to stdout")
                .short('b')
                .action(ArgAction::SetTrue)
                .long("batch"),
        )
        // Beep option
        //     [-B,--beep]
        .arg(
            Arg::new("beep")
                .help("beep if command has a change result")
                .short('B')
                .action(ArgAction::SetTrue)
                .long("beep"),
        )
        // exit on change option
        //     [-g,--chgexit[=COUNT]]
        .arg(
            Arg::new("chgexit")
                .help("exit when output changes. With no value, exits after the first change; with N, exits after N changes")
                .short('g')
                .long("chgexit")
                .num_args(0..=1)
                .default_missing_value("1")
                .value_parser(clap::value_parser!(u32)),
        )
        // border option
        //     [--border]
        .arg(
            Arg::new("border")
                .help("Surround each pane with a border frame")
                .action(ArgAction::SetTrue)
                .long("border"),
        )
        // scrollbar option
        //     [--with-scrollbar]
        .arg(
            Arg::new("with_scrollbar")
                .help("When the border option is enabled, display scrollbar on the right side of watch pane.")
                .action(ArgAction::SetTrue)
                .long("with-scrollbar"),
        )
        // mouse option
        //     [--mouse]
        .arg(
            Arg::new("mouse")
                .help("enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.")
                .action(ArgAction::SetTrue)
                .long("mouse"),
        )
        // Enable ANSI color option
        //     [-c,--color]
        .arg(
            Arg::new("color")
                .help("interpret ANSI color and style sequences")
                .short('c')
                .action(ArgAction::SetTrue)
                .long("color"),
            )
        // Enable Reverse mode option
        //     [-r,--reverse]
        .arg(
            Arg::new("reverse")
                .help("display text upside down.")
                .short('r')
                .action(ArgAction::SetTrue)
                .long("reverse"),
        )
        // Compress data in memory option.
        //     [-C,--compress]
        .arg(
            Arg::new("compress")
                .help("Compress data in memory. Note: If the output of the command is small, you may not get the desired effect.")
                .short('C')
                .action(ArgAction::SetTrue)
                .long("compress"),
        )
        // no-title flag.
        //     [--no-title]
        .arg(
            Arg::new("no_title")
                .help("hide the UI on start. Use `t` to toggle it.")
                .long("no-title")
                .action(ArgAction::SetTrue)
                .short('t'),
        )
        // enable charcter summary option
        .arg(
            Arg::new("enable_summary_char")
                .help("collect character-level diff count in summary.")
                .long("enable-summary-char")
                .action(ArgAction::SetTrue),
        )
        // Enable line number mode option
        //   [--line-number,-N]
        .arg(
            Arg::new("line_number")
                .help("show line number")
                .short('N')
                .action(ArgAction::SetTrue)
                .long("line-number"),
        )
        // Disable wrap mode option
        //   [--wrap,-w]
        .arg(
            Arg::new("wrap")
            .help("disable line wrap mode")
                .short('w')
                .action(ArgAction::SetTrue)
                .long("wrap"),
        )
        // help banner disable flag.
        //     [--no-help-banner]
        .arg(
            Arg::new("no_help_banner")
                .help("hide the \"Display help with h key\" message")
                .long("no-help-banner")
                .action(ArgAction::SetTrue),
        )
        // summary disable flag.
        //     [--no-summary]
        .arg(
            Arg::new("no_summary")
                .help("disable the calculation for summary that is running behind the scenes, and disable the summary function in the first place.")
                .long("no-summary")
                .action(ArgAction::SetTrue),
        )
        // completion output option
        //     [--completion]
        .arg(
            Arg::new("completion")
                .help("Output shell completion script")
                .long("completion")
                .value_name("SHELL")
                .value_parser(["bash", "fish", "zsh"])
                .action(ArgAction::Set)
                .exclusive(true),
        )
        // exec flag.
        //     [-x,--exec]
        .arg(
            Arg::new("exec")
                .help("Run the command directly, not through the shell. Much like the `-x` option of the watch command.")
                .short('x')
                .action(ArgAction::SetTrue)
                .long("exec"),

        )
        // use pty flag.
        //     [-p,--use-pty]
        .arg(
            Arg::new("use_pty")
                .help("Run the command through a pseudo-TTY so commands that colorize on terminals can keep color output.")
                .long("use-pty")
                .short('p')
                .action(ArgAction::SetTrue),
        )
        // output only flag.
        //     [-O,--diff-output-only]
        .arg(
            Arg::new("diff_output_only")
                .help("Display only the lines with differences during `line` diff and `word` diff.")
                .short('O')
                .long("diff-output-only")
                .requires("differences")
                .action(ArgAction::SetTrue),

        )

        // -- options --
        // Option to specify the command to be executed when the output fluctuates.
        //     [-A,--aftercommand]
        .arg(
            Arg::new("after_command")
                .help("Executes the specified command if the output changes. Information about changes is stored in json format in environment variable ${HWATCH_DATA}.")
                .short('A')
                .long("aftercommand")
                .value_hint(ValueHint::CommandString)
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("after_command_result_write_file")
                .help("TODO: あとでかく")
                .long("after-command-result-write-file")
                .requires("after_command")
                .action(ArgAction::SetTrue),
        )
        // Logging option
        //   [--logfile,-l] /path/to/logfile
        // ex.)
        //      {timestamp: "...", command: "....", output: ".....", ...}
        //      {timestamp: "...", command: "....", output: ".....", ...}
        //      {timestamp: "...", command: "....", output: ".....", ...}
        .arg(
            Arg::new("logfile")
                .help("logging file. if a log file is already used, its contents will be read and executed.")
                .short('l')
                .long("logfile")
                .num_args(0..=1)
                .value_hint(ValueHint::FilePath)
                .action(ArgAction::Append),
        )
        // shell command
        //   [--shell,-s] command
        .arg(
            Arg::new("shell_command")
                .help("shell to use at runtime. can also insert the command to the location specified by {COMMAND}.")
                .short('s')
                .long("shell")
                .action(ArgAction::Append)
                .value_hint(ValueHint::CommandString)
                .default_value(SHELL_COMMAND),
        )
        // Interval option
        //   [--interval,-n] second(default:2)
        .arg(
            Arg::new("interval")
                .help("seconds to wait between updates")
                .short('n')
                .long("interval")
                // .action(ArgAction::Append)
                .num_args(1)
                .value_parser(clap::value_parser!(f64))
                .default_value("2"),
        )
        // Precise Interval Mode
        //   [--precise] default:false
        .arg(
            Arg::new("precise")
                .help("Attempt to run as close to the interval as possible, regardless of how long the command takes to run")
                .long("precise")
                .action(ArgAction::SetTrue)
        )
        // set limit option
        //   [--limit,-L] size(default:5000)
        .arg(
            Arg::new("limit")
                .help("Set the number of history records to keep. only work in watch mode. Set `0` for unlimited recording.")
                .short('L')
                .long("limit")
                .value_parser(clap::value_parser!(u32))
                .default_value(HISTORY_LIMIT),
        )
        // tab size set option
        //   [--tab_size] size(default:4)
        .arg(
            Arg::new("tab_size")
                .help("Specifying tab display size")
                .long("tab-size")
                .value_parser(clap::value_parser!(u16))
                .action(ArgAction::Append)
                .default_value("4"),
        )
        // Enable diff mode option
        //   [--differences,-d] [none, watch, line, word]
        // NOTE: `normalize_args` is preprocessed so that `watch` is selected if no value is set for this option.
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
        // Set output mode option
        //   [--output,-o] [output, stdout, stderr]
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
        // Set keymap option
        //   [--keymap,-K] keymap(shortcut=function)
        .arg(
            Arg::new("keymap")
                .help("Add keymap")
                .short('K')
                .long("keymap")
                .action(ArgAction::Append),
        )
}

/// Normalize options in args.
/// This function is needed to allow users to specify diff mode options without explicitly providing a mode, defaulting to "watch" mode if the mode is not specified.
fn collect_known_diff_mode_names(args: &[OsString]) -> HashSet<String> {
    let mut modes = HashSet::from([
        "none".to_string(),
        "watch".to_string(),
        "line".to_string(),
        "word".to_string(),
    ]);

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_string_lossy();
        let plugin_path = if arg == "--diff-plugin" {
            index += 1;
            args.get(index).map(|value| value.to_string_lossy().into_owned())
        } else {
            arg.strip_prefix("--diff-plugin=").map(|value| value.to_string())
        };

        if let Some(plugin_path) = plugin_path {
            if let Ok(registration) = plugin_diffmode::load_plugin(Path::new(&plugin_path)) {
                modes.insert(registration.name);
            }
        }

        index += 1;
    }

    modes
}

fn normalize_args(args: Vec<OsString>) -> Vec<OsString> {
    let known_diff_modes = collect_known_diff_mode_names(&args);
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
                    if known_diff_modes.contains(next_value.as_ref()) {
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

    return normalized;
}

fn get_clap_matcher(cmd_app: Command) -> clap::ArgMatches {
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

fn output_completion(shell: &str) -> bool {
    match shell {
        "bash" => {
            print!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/completion/bash/hwatch-completion.bash"
                ))
            );
            true
        }
        "fish" => {
            print!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/completion/fish/hwatch.fish"
                ))
            );
            true
        }
        "zsh" => {
            print!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/completion/zsh/_hwatch"
                ))
            );
            true
        }
        _ => false,
    }
}

fn parse_completion_from_cli() -> Result<Option<String>, String> {
    let mut args = std::env::args_os();
    let _ = args.next();
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        if arg == "--" {
            break;
        }

        let Some(arg_str) = arg.to_str() else {
            continue;
        };

        if arg_str == "--completion" {
            let Some(value) = iter.next() else {
                return Err("missing value for --completion".to_string());
            };
            return Ok(Some(value.to_string_lossy().into_owned()));
        }

        if let Some(value) = arg_str.strip_prefix("--completion=") {
            return Ok(Some(value.to_string()));
        }
    }

    Ok(None)
}

fn main() {
    match parse_completion_from_cli() {
        Ok(Some(shell)) => {
            if !output_completion(&shell) {
                eprintln!("unknown shell for --completion: {shell}");
                std::process::exit(2);
            }
            return;
        }
        Ok(None) => {}
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    }

    // Get command args matches
    let mut cmd_app = build_app();
    let matcher = get_clap_matcher(cmd_app.clone());

    // Get options flag
    let batch = matcher.get_flag("batch");
    let compress = matcher.get_flag("compress");
    let precise = matcher.get_flag("precise");
    let exit_on_change = matcher.get_one::<u32>("chgexit").copied();

    // Get after command
    let after_command = matcher.get_one::<String>("after_command");
    let after_command_result_write_file = matcher.get_flag("after_command_result_write_file");

    // Get logfile
    let logfile = matcher.get_one::<String>("logfile");

    // check _logfile directory
    // TODO(blacknon): commonに移す？(ここで直書きする必要性はなさそう)
    let mut load_results = vec![];
    if let Some(logfile) = logfile {
        // logging log
        let log_path = Path::new(logfile);
        let log_dir = log_path.parent().unwrap();
        let cur_dir = std::env::current_dir().expect("cannot get current directory");
        let abs_log_path = cur_dir.join(log_path);
        let abs_log_dir = cur_dir.join(log_dir);

        // check _log_dir exist
        if !abs_log_dir.exists() {
            let err = cmd_app.error(
                ErrorKind::ValueValidation,
                format!("directory {abs_log_dir:?} is not exists."),
            );
            err.exit();
        }

        // load logfile
        match load_logfile(abs_log_path.to_str().unwrap(), compress) {
            Ok(results) => {
                load_results = results;
            }
            Err(e) => {
                let mut is_overwrite_question = false;
                match e {
                    common::LoadLogfileError::LogfileEmpty => {
                        eprintln!("file {abs_log_path:?} is exists and empty.");
                        is_overwrite_question = true;
                    }
                    common::LoadLogfileError::LoadFileError(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => {}
                        _ => {
                            eprintln!("file {abs_log_path:?} is exists and load error.");
                            eprintln!("{err:?}");
                            is_overwrite_question = true;
                        }
                    },
                    common::LoadLogfileError::JsonParseError(err) => {
                        eprintln!("file {abs_log_path:?} is exists and json parse error.");
                        eprintln!("{err:?}");
                        is_overwrite_question = true;
                    }
                }

                if is_overwrite_question {
                    let answer = Question::new("Log to the same file?")
                        .default(Answer::YES)
                        .show_defaults()
                        .confirm();

                    if answer != Answer::YES {
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    // Create channel
    let (tx, rx) = unbounded();

    // interval
    let shared_interval: SharedInterval = match matcher.get_one::<f64>("interval") {
        Some(override_interval) => SharedInterval::new(RunInterval::new(*override_interval).into()),
        None => SharedInterval::default(),
    };

    // history limit
    let default_limit: u32 = HISTORY_LIMIT.parse().unwrap();
    let limit = matcher.get_one::<u32>("limit").unwrap_or(&default_limit);

    // tab size
    let tab_size = *matcher
        .get_one::<u16>("tab_size")
        .unwrap_or(&DEFAULT_TAB_SIZE);

    // enable summary char
    let enable_summary_char = matcher.get_flag("enable_summary_char");

    // output mode
    let output_mode = match matcher.get_one::<String>("output").unwrap().as_str() {
        "output" => common::OutputMode::Output,
        "stdout" => common::OutputMode::Stdout,
        "stderr" => common::OutputMode::Stderr,
        _ => common::OutputMode::Output,
    };

    // Get Add keymap
    let keymap_options: Vec<&str> = matcher
        .get_many::<String>("keymap")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    // Parse Add Keymap
    let keymap = match keymap::generate_keymap(keymap_options) {
        Ok(keymap) => keymap,
        _ => {
            eprintln!("Failed to parse keymap.");
            std::process::exit(1);
        }
    };

    let watch_diff_fg = match std::env::var("HWATCH_WATCH_FG") {
        Ok(value) => match common::parse_ansi_color(&value) {
            Ok(color) => Some(color),
            Err(message) => {
                let err = cmd_app.error(
                    ErrorKind::ValueValidation,
                    format!("$HWATCH_WATCH_FG {message}"),
                );
                err.exit();
            }
        },
        Err(_) => None,
    };

    let watch_diff_bg = match std::env::var("HWATCH_WATCH_BG") {
        Ok(value) => match common::parse_ansi_color(&value) {
            Ok(color) => Some(color),
            Err(message) => {
                let err = cmd_app.error(
                    ErrorKind::ValueValidation,
                    format!("$HWATCH_WATCH_BG {message}"),
                );
                err.exit();
            }
        },
        Err(_) => None,
    };

    // set command
    let command_line: Vec<String>;
    if let Some(value) = matcher.get_many::<String>("command") {
        command_line = value.into_iter().cloned().collect();
    } else {
        // check load_results
        if load_results.is_empty() {
            let err = cmd_app.error(
                ErrorKind::InvalidValue,
                "command not specified and logfile is empty.".to_string(),
            );
            err.exit();
        }

        // set command
        let command = load_results.last().unwrap().command.clone();
        command_line = shell_words::split(&command).unwrap();
    }

    // Start Command Thread
    {
        let m = matcher.clone();
        let tx = tx.clone();
        let shell_command = m.get_one::<String>("shell_command").unwrap().to_string();
        let command: Vec<_> = command_line;
        let is_exec = m.get_flag("exec");
        let is_pty = m.get_flag("use_pty");
        let run_interval_ptr = shared_interval.clone();
        let _ = thread::spawn(move || loop {
            let run_interval = run_interval_ptr.read().expect("Non poisoned block");
            let paused = run_interval.paused;
            let interval = run_interval.interval;
            drop(run_interval); // We manually drop here or else it locks anything else from reading/writing the interval
            let mut time_to_sleep: f64 = interval;

            if !paused {
                // Create cmd..
                let mut exe = exec::ExecuteCommand::new(tx.clone());

                // Set shell command
                exe.shell_command = shell_command.clone();

                // Set command
                exe.command = command.clone();

                // Set compress
                exe.is_compress = compress;

                // Set is exec flag.
                exe.is_exec = is_exec;
                exe.is_pty = is_pty;

                let before_start = SystemTime::now();
                // Exec command
                exe.exec_command();

                if precise {
                    let elapsed: f64 = SystemTime::now()
                        .duration_since(before_start)
                        .unwrap_or_default()
                        .as_secs_f64();
                    time_to_sleep = match elapsed > time_to_sleep {
                        true => 0_f64,
                        false => time_to_sleep - elapsed,
                    };
                }
            }

            std::thread::sleep(Duration::from_secs_f64(time_to_sleep));
        });
    }

    let mut diff_mode_name_to_index: HashMap<String, usize> = HashMap::new();

    // create diff_mode(plane)
    let diff_mode_plane = diffmode_plane::DiffModeAtPlane::new();

    // create diff_mode(watch)
    let diff_mode_watch = diffmode_watch::DiffModeAtWatch::new();

    // create diff_mode(line)
    let mut diff_mode_line = diffmode_line::DiffModeAtLineDiff::new();
    diff_mode_line.is_word_highlight = false;

    // create diff_mode(word)
    let mut diff_mode_word = diffmode_line::DiffModeAtLineDiff::new();
    diff_mode_word.is_word_highlight = true;

    // set diff_modes
    let mut diff_modes: Vec<Arc<Mutex<Box<dyn DiffMode>>>> = vec![
        Arc::new(Mutex::new(Box::new(diff_mode_plane))),
        Arc::new(Mutex::new(Box::new(diff_mode_watch))),
        Arc::new(Mutex::new(Box::new(diff_mode_line))),
        Arc::new(Mutex::new(Box::new(diff_mode_word))),
    ];

    for (index, name) in ["none", "watch", "line", "word"].into_iter().enumerate() {
        diff_mode_name_to_index.insert(name.to_string(), index);
    }

    if let Some(plugin_paths) = matcher.get_many::<String>("diff_plugin") {
        for plugin_path in plugin_paths {
            let plugin_path = Path::new(plugin_path);
            let registration = match plugin_diffmode::load_plugin(plugin_path) {
                Ok(registration) => registration,
                Err(message) => {
                    let err = cmd_app.error(ErrorKind::Io, message);
                    err.exit();
                }
            };

            let plugin_name = registration.name;
            if diff_mode_name_to_index.contains_key(&plugin_name) {
                let err = cmd_app.error(
                    ErrorKind::ArgumentConflict,
                    format!("duplicate diff mode name: '{plugin_name}'"),
                );
                err.exit();
            }

            let index = diff_modes.len();
            diff_modes.push(Arc::new(Mutex::new(registration.mode)));
            diff_mode_name_to_index.insert(plugin_name, index);
        }
    }

    // diff mode
    let diff_mode = if matcher.contains_id("differences") {
        let requested = matcher.get_one::<String>("differences").unwrap();
        match diff_mode_name_to_index.get(requested) {
            Some(index) => *index,
            None => {
                let available = diff_mode_name_to_index
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let err = cmd_app.error(
                    ErrorKind::InvalidValue,
                    format!("unknown diff mode '{requested}'. Available: {available}"),
                );
                err.exit();
            }
        }
    } else {
        0
    };

    let diff_mode_width = calculate_diff_mode_header_width(&diff_modes);

    // check batch mode
    if !batch {
        // On Windows, Ctrl+C can be delivered as a process signal before key input is read.
        // Convert it into the same key event as the default keymap (`ctrl-c=cancel`).
        let tx_ctrlc = tx.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = tx_ctrlc.send(event::AppEvent::TerminalEvent(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })));
        });

        // is watch mode
        // Create view
        let mut view = view::View::new(shared_interval.clone(), diff_modes)
            .set_tab_size(tab_size)
            .set_limit(*limit)
            .set_beep(matcher.get_flag("beep"))
            .set_exit_on_change(exit_on_change)
            .set_border(matcher.get_flag("border"))
            .set_scroll_bar(matcher.get_flag("with_scrollbar"))
            .set_mouse_events(matcher.get_flag("mouse"))
            // set keymap
            .set_keymap(keymap)
            // Set color in view
            .set_color(matcher.get_flag("color"))
            // Set watch diff highlight colors
            .set_watch_diff_colors(watch_diff_fg, watch_diff_bg)
            // Set line number in view
            .set_line_number(matcher.get_flag("line_number"))
            // Set reverse mode in view
            .set_reverse(matcher.get_flag("reverse"))
            // Set wrap mode in view
            .set_wrap_mode(!matcher.get_flag("wrap"))
            // Set output in view
            .set_output_mode(output_mode)
            // Set diff(watch diff) in view
            .set_diff_mode(diff_mode)
            .set_diff_mode_width(diff_mode_width)
            .set_only_diffline(matcher.get_flag("diff_output_only"))
            // Set enable summary char
            .set_enable_summary_char(enable_summary_char)
            .set_show_ui(!matcher.get_flag("no_title"))
            .set_show_help_banner(!matcher.get_flag("no_help_banner"));

        // Set logfile
        if let Some(logfile) = logfile {
            view = view.set_logfile(logfile.to_string());
        }

        // Set after_command
        if let Some(after_command) = after_command {
            view = view.set_after_command(after_command.to_string());
            view = view.set_after_command_result_write_file(after_command_result_write_file);
        }

        // start app.
        let _res = view.start(tx, rx, load_results);
    } else {
        // is batch mode
        let mut batch = batch::Batch::new(rx, diff_modes)
            .set_beep(matcher.get_flag("beep"))
            .set_color(matcher.get_flag("color"))
            .set_exit_on_change(exit_on_change)
            .set_output_mode(output_mode)
            .set_diff_mode(diff_mode)
            .set_line_number(matcher.get_flag("line_number"))
            .set_reverse(matcher.get_flag("reverse"))
            .set_only_diffline(matcher.get_flag("diff_output_only"));

        // Set logfile
        if let Some(logfile) = logfile {
            batch = batch.set_logfile(logfile.to_string());
        }

        // Set after_command
        if let Some(after_command) = after_command {
            batch = batch.set_after_command(after_command.to_string());
            batch = batch.set_after_command_result_write_file(after_command_result_write_file);
        }

        // start batch.
        let _res = batch.run();
    }
}

fn calculate_diff_mode_header_width(diff_modes: &[Arc<Mutex<Box<dyn DiffMode>>>]) -> usize {
    let mut max_width = 0;

    for diff_mode in diff_modes {
        let mut diff_mode = diff_mode.lock().unwrap();
        for only_diffline in [false, true] {
            let mut options = hwatch_diffmode::DiffModeOptions::new();
            options.set_only_diffline(only_diffline);
            diff_mode.set_option(options);
            let header_text = diff_mode.get_header_text();
            max_width = max_width.max(UnicodeWidthStr::width(header_text.as_str()));
        }
    }

    max_width
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_run_interval() {
        let mut actual = RunInterval::default();
        assert!(!actual.paused);
        assert_eq!(actual.interval, 2.0);
        actual.increase(1.5);
        actual.toggle_pause();
        assert!(actual.paused);
        assert_eq!(actual.interval, 3.5);
        actual.decrease(0.5);
        actual.toggle_pause();
        assert!(!actual.paused);
        assert_eq!(actual.interval, 3.0);
    }

    #[test]
    /// Test that decreasing the interval does not go below zero and does not cause underflow.
    fn run_interval_decrease_does_not_go_below_threshold() {
        let mut actual = RunInterval::new(1.0);
        actual.decrease(1.0);
        actual.decrease(2.0);

        assert_eq!(actual.interval, 1.0);
        assert!(!actual.paused);
    }

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
}
