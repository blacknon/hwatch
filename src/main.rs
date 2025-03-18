// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.19
// TODO(blacknon): watchウィンドウの表示を折り返しだけではなく、横方向にスクロールして出力するモードも追加する(un wrap mode)
//                 [[FR] Disable line wrapping #182](https://github.com/blacknon/hwatch/issues/182)
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): DiffModeをInterfaceで取り扱うようにし、historyへの追加や検索時のhitなどについてもInterface側で取り扱えるようにする。
//                 - DiffModeのPlugin化の布石としての対応
//                   - これができたら、数字ごとの差分をわかりやすいように表示させたり、jsonなどの形式が決まってる場合にはそこだけdiffさせるような仕組みにも簡単に対応できると想定
// TODO(blacknon): Github Actionsをきれいにする

// v0.3.xx
// TODO(blacknon): [FR: add "completion" subcommand](https://github.com/blacknon/hwatch/issues/107)
// TODO(blacknon): filter modeのハイライト表示の色を環境変数で定義できるようにする
// TODO(blacknon): filter modeの検索ヒット数を表示する(どうやってやろう…？というより、どこに表示させよう…？)
// TODO(blacknon): Windowsのバイナリをパッケージマネジメントシステムでインストール可能になるよう、Releaseでうまいこと処理をする
// TODO(blacknon): UTF-8以外のエンコードでも動作するよう対応する(エンコード対応)
// TODO(blacknon): 空白の数だけ違う場合、diffとして扱わないようにするオプションの追加(shortcut keyではなく、`:set hogehoge...`で指定する機能として実装)
// TODO(blacknon): watchをモダンよりのものに変更する
// TODO(blacknon): diff modeをさらに複数用意し、選択・切り替えできるdiffをオプションから指定できるようにする(watchをold-watchにして、モダンなwatchをデフォルトにしたり)
// TODO(blacknon): formatを整える機能や、diff時に特定のフォーマットかどうかで扱いを変える機能について、追加する方法を模索する(プラグインか、もしくはパイプでうまいこときれいにする機能か？)

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
extern crate termwiz;
extern crate tokio;
extern crate unicode_segmentation;
extern crate unicode_width;

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
use common::{load_logfile, DiffMode};
use crossbeam_channel::unbounded;
use question::{Answer, Question};
use std::env::args;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

// local modules
mod ansi;
mod app;
mod batch;
mod common;
mod errors;
mod event;
mod exec;
mod exit;
mod header;
mod help;
mod history;
mod keymap;
mod output;
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
        .trailing_var_arg(true)
        .author(crate_authors!())

        // -- command --
        .arg(
            Arg::new("command")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .num_args(0..)
                .value_hint(ValueHint::CommandWithArguments)
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
        //
        // exec flag.
        //     [-x,--exec]
        .arg(
            Arg::new("exec")
                .help("Run the command directly, not through the shell. Much like the `-x` option of the watch command.")
                .short('x')
                .action(ArgAction::SetTrue)
                .long("exec"),

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
                .action(ArgAction::Append)
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
                .help("Set the number of history records to keep. only work in watch mode. Set `0` for unlimited recording. (default: 5000)")
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
        .arg(
            Arg::new("differences")
                .help("highlight changes between updates")
                .long("differences")
                .short('d')
                .num_args(0..=1)
                .value_parser(["none", "watch", "line", "word"])
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

fn get_clap_matcher(cmd_app: Command) -> clap::ArgMatches {
    let env_config = std::env::var("HWATCH").unwrap_or_default();
    let env_args: Vec<&str> = env_config.split_ascii_whitespace().collect();
    let mut os_args = std::env::args_os();
    let mut args: Vec<std::ffi::OsString> = vec![];

    // First argument is the program name
    args.push(os_args.next().unwrap());

    // Environment variables go next so that they can be overridded
    // TODO: Currently, the opposites of command-line options are not
    // yet implemented. E.g., there is no `--no-color` to override
    // `--color` in the HWATCH environment variable.
    args.extend(env_args.iter().map(std::ffi::OsString::from));
    args.extend(os_args);

    cmd_app.get_matches_from(args)
}

fn main() {
    // Get command args matches
    let mut cmd_app = build_app();
    let matcher = get_clap_matcher(cmd_app.clone());

    // Get options flag
    let batch = matcher.get_flag("batch");
    let compress = matcher.get_flag("compress");
    let precise = matcher.get_flag("precise");

    // Get after command
    let after_command = matcher.get_one::<String>("after_command");

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

    // diff mode
    let diff_mode = if matcher.contains_id("differences") {
        match matcher.get_one::<String>("differences").unwrap().as_str() {
            "none" => DiffMode::Disable,
            "watch" => DiffMode::Watch,
            "line" => DiffMode::Line,
            "word" => DiffMode::Word,
            _ => DiffMode::Disable,
        }
    } else {
        DiffMode::Disable
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

    // set command
    let command_line: Vec<String>;
    if let Some(value) = matcher.get_many::<String>("command") {
        command_line = value.into_iter().map(|s| s.clone()).collect()
    } else {
        // check load_results
        if load_results.is_empty() {
            let err = cmd_app.error(
                ErrorKind::InvalidValue,
                format!("command not specified and logfile is empty."),
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
        let run_interval_ptr = shared_interval.clone();
        let _ = thread::spawn(move || loop {
            let run_interval = run_interval_ptr.read().expect("Non poisoned block");
            let paused = run_interval.paused.clone();
            let interval = run_interval.interval.clone();
            drop(run_interval); // We manually drop here or else it locks anything else from reading/writing the interval
            let mut time_to_sleep: f64 = interval;

            if paused == false {
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

    // check batch mode
    if !batch {
        // is watch mode
        // Create view
        let mut view = view::View::new(shared_interval.clone())
            .set_tab_size(tab_size)
            .set_limit(*limit)
            .set_beep(matcher.get_flag("beep"))
            .set_border(matcher.get_flag("border"))
            .set_scroll_bar(matcher.get_flag("with_scrollbar"))
            .set_mouse_events(matcher.get_flag("mouse"))
            // set keymap
            .set_keymap(keymap)
            // Set color in view
            .set_color(matcher.get_flag("color"))
            // Set line number in view
            .set_line_number(matcher.get_flag("line_number"))
            // Set reverse mode in view
            .set_reverse(matcher.get_flag("reverse"))
            // Set output in view
            .set_output_mode(output_mode)
            // Set diff(watch diff) in view
            .set_diff_mode(diff_mode)
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
        }

        // start app.
        let _res = view.start(tx, rx, load_results);
    } else {
        // is batch mode
        let mut batch = batch::Batch::new(rx)
            .set_beep(matcher.get_flag("beep"))
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
        }

        // start batch.
        let _res = batch.run();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_interval() {
        let mut actual = RunInterval::default();
        assert_eq!(actual.paused, false);
        assert_eq!(actual.interval, 2.0);
        actual.increase(1.5);
        actual.toggle_pause();
        assert_eq!(actual.paused, true);
        assert_eq!(actual.interval, 3.5);
        actual.decrease(0.5);
        actual.toggle_pause();
        assert_eq!(actual.paused, false);
        assert_eq!(actual.interval, 3.0);
    }
}
