// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.14
// TODO(blacknon): キー入力のカスタマイズが行えるようにする(custom keymap)

// v0.3.15
// TODO(blacknon): 終了時にYes/Noで確認を取る機能を実装する(オプションで無効化させる)
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)

// v0.3.16
// TODO(blacknon): https://github.com/blacknon/hwatch/issues/101
//                 - ログを読み込ませて、そのまま続きの処理を行わせる機能の追加

// v1.0.0
// TODO(blacknon): vimのように内部コマンドを利用した表示切り替え・出力結果の編集機能を追加する
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能にする
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man
// TODO(blacknon): ライフタイムの名称をちゃんと命名する。
// TODO(blacknon): エラーなどのメッセージ表示領域の作成
// TODO(blacknon): diffのライブラリをsimilarに切り替える？
//                 - https://github.com/mitsuhiko/similar
//                 - 目的としては、複数文字を区切り文字指定して差分のある箇所をもっとうまく抽出できるようにしてやりたい、というもの
//                 - diffのとき、スペースの増減は無視するようなオプションがほしい(あるか？というのは置いといて…)


// crate
// extern crate ansi_parser;
extern crate ansi_parser;
extern crate ansi_term;
extern crate async_std;
extern crate chrono;
extern crate crossbeam_channel;
extern crate crossterm;
extern crate ctrlc;
extern crate difference;
extern crate futures;
extern crate heapless;
extern crate question;
extern crate regex;
extern crate serde;
extern crate shell_words;
extern crate termwiz;
extern crate ratatui as tui;

// macro crate
#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

// modules
use clap::{Arg, ArgAction, Command, ValueHint, builder::ArgPredicate};
use question::{Answer, Question};
use std::env::args;
use std::path::Path;
use std::sync::{Arc, RwLock};
use crossbeam_channel::unbounded;
use std::thread;
use std::time::Duration;
use common::DiffMode;

// local modules
mod ansi;
mod app;
mod batch;
mod common;
mod event;
mod exec;
mod header;
mod help;
mod history;
mod keys;
mod output;
mod view;
mod watch;

// const
pub const DEFAULT_INTERVAL: f64 = 2.0;
pub const DEFAULT_TAB_SIZE: u16 = 4;
pub const HISTORY_WIDTH: u16 = 25;
pub const SHELL_COMMAND_EXECCMD: &str = "{COMMAND}";
type Interval = Arc<RwLock<f64>>;

// const at Windows
#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(windows)]
const SHELL_COMMAND: &str = "cmd /C";

// const at not Windows
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";
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
                .required(true),
        )

        // -- flags --
        // Enable batch mode option
        //     [-b,--batch]
        .arg(
            Arg::new("batch")
                .help("output exection results to stdout")
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
        // exec flag.
        //     [--no-title]
        .arg(
            Arg::new("no_title")
                .help("hide the UI on start. Use `t` to toggle it.")
                .long("no-title")
                .action(ArgAction::SetTrue)
                .short('t'),
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
        // exec flag.
        //     [--no-help-banner]
        .arg(
            Arg::new("no_help_banner")
                .help("hide the \"Display help with h key\" message")
                .long("no-help-banner")
                .action(ArgAction::SetTrue),
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
        // output only flag.
        //     [-O,--diff-output-only]
        .arg(
            Arg::new("diff_output_only")
                .help("Display only the lines with differences during line diff and word diff.")
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
                .help("logging file")
                .short('l')
                .long("logfile")
                .value_hint(ValueHint::FilePath)
                .action(ArgAction::Append),
        )
        // shell command
        //   [--shell,-s] command
        .arg(
            Arg::new("shell_command")
                .help("shell to use at runtime. can  also insert the command to the location specified by {COMMAND}.")
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
}

fn get_clap_matcher() -> clap::ArgMatches {
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

    build_app().get_matches_from(args)
}

fn main() {
    // Get command args matches
    let matcher = get_clap_matcher();

    // Get options flag
    let batch = matcher.get_flag("batch");

    let after_command = matcher.get_one::<String>("after_command");
    let logfile = matcher.get_one::<String>("logfile");

    // check _logfile directory
    // TODO(blacknon): commonに移す？(ここで直書きする必要性はなさそう)
    if let Some(logfile) = logfile {
        let _log_path = Path::new(logfile);
        let _log_dir = _log_path.parent().unwrap();
        let _cur_dir = std::env::current_dir().expect("cannot get current directory");
        let _abs_log_path = _cur_dir.join(_log_path);
        let _abs_log_dir = _cur_dir.join(_log_dir);

        // check _log_path exist
        if _abs_log_path.exists() {
            println!("file {_abs_log_path:?} is exists.");
            let answer = Question::new("Log to the same file?")
                .default(Answer::YES)
                .show_defaults()
                .confirm();

            if answer != Answer::YES {
                std::process::exit(1);
            }
        }

        // check _log_dir exist
        if !_abs_log_dir.exists() {
            println!("directory {_abs_log_dir:?} is not exists.");
            std::process::exit(1);
        }
    }

    // Create channel
    let (tx, rx) = unbounded();

    // interval
    let override_interval: f64 = *matcher.get_one::<f64>("interval").unwrap_or(&DEFAULT_INTERVAL);
    let interval = Interval::new(override_interval.into());

    // tab size
    let tab_size = *matcher.get_one::<u16>("tab_size").unwrap_or(&DEFAULT_TAB_SIZE);

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

    // Start Command Thread
    {
        let m = matcher.clone();
        let tx = tx.clone();
        let shell_command = m.get_one::<String>("shell_command").unwrap().to_string();
        let command: Vec<_> = m.get_many::<String>("command").unwrap().into_iter().map(|s| s.clone()).collect();
        let is_exec = m.get_flag("exec");
        let interval = interval.clone();
        let _ = thread::spawn(move || loop {
            // Create cmd..
            let mut exe = exec::ExecuteCommand::new(tx.clone());

            // Set shell command
            exe.shell_command = shell_command.clone();

            // Set command
            exe.command = command.clone();

            // Set is exec flag.
            exe.is_exec = is_exec;

            // Exec command
            exe.exec_command();

            let sleep_interval = *interval.read().unwrap();
            std::thread::sleep(Duration::from_secs_f64(sleep_interval));
        });
    }

    // check batch mode
    if !batch {
        // is watch mode
        // Create view
        let mut view = view::View::new(interval.clone())
            // Set interval on view.header
            .set_interval(interval)
            .set_tab_size(tab_size)
            .set_beep(matcher.get_flag("beep"))
            .set_mouse_events(matcher.get_flag("mouse"))

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
        let _res = view.start(tx, rx);
    } else {
        // is batch mode
        let mut batch = batch::Batch::new(tx, rx)
            .set_beep(matcher.get_flag("beep"))
            .set_output_mode(output_mode)
            .set_diff_mode(diff_mode)
            .set_line_number(matcher.get_flag("line_number"))
            .set_reverse(matcher.get_flag("reverse"))
            .set_only_diffline(matcher.get_flag("diff_output_only"));

        // Set after_command
        if let Some(after_command) = after_command {
            batch = batch.set_after_command(after_command.to_string());
        }

        // start batch.
        let _res = batch.run();
    }
}
