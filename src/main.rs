// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.12
// TODO(blacknon): pagedown/pageupスクロールの実装
// TODO(blacknon): scrollで一番↓まで行くとき、ページの一番下がターミナルの最終行になるように変更する
// TODO(blacknon): issueの中で簡単に実装できそうなやつ

// v0.3.13
// TODO(blakcnon): batch modeの実装.
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能にする
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man
// TODO(blacknon): errorとの比較を行わない(正常終了時のみを比較対象とし、errorの履歴をスキップしてdiffする)キーバインドの追加(なんかのmode?)
//                 => outputごとに分離して比較できる仕組みにする方式で対処？
// TODO(blacknon): ライフタイムの名称をちゃんと命名する。
// TODO(blacknon): エラーなどのメッセージ表示領域の作成
// TODO(blacknon): diffのライブラリをsimilarに切り替える？
//                 - https://github.com/mitsuhiko/similar
//                 - 目的としては、複数文字を区切り文字指定して差分のある箇所をもっとうまく抽出できるようにしてやりたい、というもの
//                 - diffのとき、スペースの増減は無視するようなオプションがほしい(あるか？というのは置いといて…)
// TODO(blacknon): diffのとき、stdout/stderrでの比較時におけるdiffでhistoryも変化させる？
//                 - データの扱いが変わってきそう？
//                 - どっちにしてもデータがあるなら、stdout/stderrのとこだけで比較するような何かがあればいい？？？

// crate
// extern crate ansi_parser;
extern crate hwatch_ansi_parser as ansi_parser;
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
use clap::{AppSettings, Arg, Command};
use question::{Answer, Question};
use std::env::args;
use std::path::Path;
use std::sync::{Arc, RwLock};
// use std::sync::mpsc::channel;
use crossbeam_channel::unbounded;
use std::thread;
use std::time::Duration;

// local modules
mod ansi;
mod app;
mod common;
mod event;
mod exec;
mod header;
mod help;
mod history;
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
fn build_app() -> clap::Command<'static> {
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
        .allow_hyphen_values(true)
        .version(crate_version!())
        .trailing_var_arg(true)
        .author(crate_authors!())
        .setting(AppSettings::DeriveDisplayOrder)

        // -- command --
        .arg(
            Arg::new("command")
                // .allow_hyphen_values(true)
                .takes_value(true)
                .allow_invalid_utf8(true)
                .multiple_values(true)
                .required(true),
        )

        // -- flags --
        // Enable batch mode option
        //     [-b,--batch]
        // .arg(
        //     Arg::with_name("batch")
        //         .help("output exection results to stdout")
        //         .short('b')
        //         .long("batch"),
        // )
        // Beep option
        //     [-B,--beep]
        .arg(
            Arg::new("beep")
                .help("beep if command has a change result")
                .short('B')
                .long("beep"),
        )
        // mouse option
        //     [--mouse]
        .arg(
            Arg::new("mouse")
                .help("enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.")
                .long("mouse"),
        )
        .arg(
            Arg::new("tab_size")
                .help("Specifying tab display size")
                .long("tab_size")
                .takes_value(true)
                .default_value("4"),
        )
        // Option to specify the command to be executed when the output fluctuates.
        //     [-C,--changed-command]
        .arg(
            Arg::new("after_command")
                .help("Executes the specified command if the output changes. Information about changes is stored in json format in environment variable ${HWATCH_DATA}.")
                .short('A')
                .long("aftercommand")
                .takes_value(true)
        )
        // Enable ANSI color option
        //     [-c,--color]
        .arg(
            Arg::new("color")
                .help("interpret ANSI color and style sequences")
                .short('c')
                .long("color"),
        )
        // Enable diff mode option
        //   [--differences,-d]
        .arg(
            Arg::new("differences")
                .help("highlight changes between updates")
                .long("differences")
                .short('d'),
        )
        .arg(
            Arg::new("no_title")
            .help("hide the UI on start. Use `t` to toggle it.")
            .long("no-title")
            .short('t'),
        )
        // Enable line number mode option
        //   [--line-number,-N]
        .arg(
            Arg::new("line_number")
                .help("show line number")
                .short('N')
                .long("line-number"),
        )
        .arg(
            Arg::new("no_help_banner")
            .help("hide the \"Display help with h key\" message")
            .long("no-help-banner")
        )

        // exec flag.
        //
        .arg(
            Arg::new("exec")
                .help("Run the command directly, not through the shell. Much like the `-x` option of the watch command.")
                .short('x')
                .long("exec"),
        )
        // -- options --
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
                .takes_value(true),
        )
        // shell command
        .arg(
            Arg::new("shell_command")
                .help("shell to use at runtime. can  also insert the command to the location specified by {COMMAND}.")
                .short('s')
                .long("shell")
                .takes_value(true)
                .default_value(SHELL_COMMAND),
        )
        // Interval option
        //   [--interval,-n] second(default:2)
        .arg(
            Arg::new("interval")
                .help("seconds to wait between updates")
                .short('n')
                .long("interval")
                .takes_value(true)
                .default_value("2"),
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
    // let batch = matcher.is_present("batch");
    let after_command = matcher.value_of("after_command");
    let logfile = matcher.value_of("logfile");
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

    let override_interval = matcher.value_of_t("interval").unwrap_or(DEFAULT_INTERVAL);
    let interval = Interval::new(override_interval.into());

    let tab_size = matcher.value_of_t("tab_size").unwrap_or(DEFAULT_TAB_SIZE);

    // Start Command Thread
    {
        let m = matcher.clone();
        let tx = tx.clone();
        let shell_command = m.value_of("shell_command").unwrap().to_string();
        let command = m.values_of_lossy("command").unwrap();
        let is_exec = m.is_present("exec");
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
    // if !batch {
    // is watch mode
    // Create view
    let mut view = view::View::new(interval.clone())
        // Set interval on view.header
        .set_interval(interval)
        .set_tab_size(tab_size)
        .set_beep(matcher.is_present("beep"))
        .set_mouse_events(matcher.is_present("mouse"))
        // Set color in view
        .set_color(matcher.is_present("color"))
        // Set line number in view
        .set_line_number(matcher.is_present("line_number"))
        // Set diff(watch diff) in view
        .set_watch_diff(matcher.is_present("differences"))
        .set_show_ui(!matcher.is_present("no_title"))
        .set_show_help_banner(!matcher.is_present("no_help_banner"));

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
    // } else {
    //     // is batch mode
    //     println!("is batch (developing now)");
    // }
}
