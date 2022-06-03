// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.6
// TODO(blakcnon): Windows対応
//                 - 文字コードを考慮に入れた設計にする(OSStringに書き換える)
// TODO(blakcnon): batch modeの実装.
// TODO(blacknon): 出力結果が変わった場合はbeepを鳴らす機能の追加
//                 watchコマンドにもある(-b, --beep)。微妙に機能としては違うものかも…？
// TODO(blacknon): 出力結果が変わった場合やコマンドの実行に失敗・成功した場合に、オプションで指定したコマンドをキックする機能を追加.
//                 - その際、環境変数をキックするコマンドに渡して実行結果や差分をキック先コマンドで扱えるようにする。
//                 - また、実行時にはシェルも指定して呼び出せるようにする？
// TODO(blacknon): ライフタイムの名称をちゃんと命名する。
// TODO(blacknon): エラーなどのメッセージ表示領域の作成
// TODO(blacknon): diffのライブラリをsimilarに切り替える？
//                 - https://github.com/mitsuhiko/similar
// TODO(blakcnon): Issue #48 の対応(https://github.com/blacknon/hwatch/issues/48).
//                 …と思ったけど、コレもしかしてshell側の問題なのでは…？

// v0.3.7
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): diffのある箇所だけを表示するモードの作成.
//                 `OnlyLine`, `OnlyWord` mode.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能にする
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man

#[warn(unused_doc_comments)]
// crate
extern crate ansi_parser;
extern crate async_std;
extern crate chrono;
extern crate crossbeam_channel;
extern crate crossterm;
extern crate difference;
extern crate futures;
extern crate heapless;
extern crate regex;
extern crate serde;
extern crate shell_words;
extern crate termwiz;
extern crate tui;

// macro crate
#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

// modules
use clap::{App, AppSettings, Arg};
use std::env::args;
use std::path::Path;
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
pub const HISTORY_WIDTH: u16 = 25;
pub const SHELL_COMMAND_EXECCMD: &'static str = "{COMMAND}";

// const at Windows
#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(windows)]
const SHELL_COMMAND: &'static str = "cmd /C";

// const at not Windows
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";
#[cfg(not(windows))]
const SHELL_COMMAND: &'static str = "sh -c";

/// Parse args and options function.
fn build_app() -> clap::App<'static, 'static> {
    // get own name
    let _program = args()
        .nth(0)
        .and_then(|s| {
            std::path::PathBuf::from(s)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap();

    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::AllowLeadingHyphen)
        // -- command --
        .arg(
            Arg::with_name("command")
                .allow_hyphen_values(true)
                .multiple(true)
                .required(true),
        )
        // -- flags --
        // Enable batch mode option
        //     [-b,--batch]
        // .arg(
        //     Arg::with_name("batch")
        //         .help("output exection results to stdout")
        //         .short("b")
        //         .long("batch"),
        // )
        // Beep option
        //     [-b,--beep]
        // Option to specify the command to be executed when the output fluctuates.
        //     [-C,--changed-command]
        // Enable ANSI color option
        //     [-c,--color]
        .arg(
            Arg::with_name("color")
                .help("interpret ANSI color and style sequences")
                .short("c")
                .long("color"),
        )
        // Enable diff mode option
        //   [--differences,-d]
        .arg(
            Arg::with_name("differences")
                .help("highlight changes between updates")
                .short("d")
                .long("differences"),
        )
        // Enable line number mode option
        //   [--line-number,-N]
        .arg(
            Arg::with_name("line_number")
                .help("show line number")
                .short("N")
                .long("line-number"),
        )
        // exec flag.
        //
        .arg(
            Arg::with_name("exec")
                .help("Run the command directly, not through the shell. Much like the `-x` option of the watch command.")
                .short("x")
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
            Arg::with_name("logfile")
                .help("logging file")
                .short("l")
                .long("logfile")
                .takes_value(true),
        )
        // shell command
        .arg(
            Arg::with_name("shell_command")
                .help("shell to use at runtime. can  also insert the command to the location specified by {COMMAND}.")
                .short("s")
                .long("shell")
                .takes_value(true)
                .default_value(SHELL_COMMAND),
        )
        // Interval option
        //   [--interval,-n] second(default:2)
        .arg(
            Arg::with_name("interval")
                .help("seconds to wait between updates")
                .short("n")
                .long("interval")
                .takes_value(true)
                .default_value("2"),
        )
}

fn main() {
    // Get command args matches
    let matche = build_app().get_matches();

    // Get options flag
    let batch = matche.is_present("batch");
    let diff = matche.is_present("differences");
    let color = matche.is_present("color");
    let is_exec = matche.is_present("exec");
    let line_number = matche.is_present("line_number");

    // Get options value
    let interval: f64 = value_t!(matche, "interval", f64).unwrap_or_else(|e| e.exit());
    // let exec = matche.value_of("exec");
    let logfile = matche.value_of("logfile");

    // check _logfile directory
    // TODO(blacknon): commonに移す？(ここで直書きする必要性はなさそう)
    if logfile != None {
        let _log_path = Path::new(logfile.clone().unwrap());
        let _log_dir = _log_path.parent().unwrap();

        // check _log_path exist
        if Path::new(_log_path).exists() {
            println!("file {:?} is exists.", _log_path);
            std::process::exit(1);
        }

        // check _log_dir exist
        if !Path::new(_log_dir).exists() {
            println!("directory {:?} is not exists.", _log_dir);
            std::process::exit(1);
        }
    }

    // Create channel
    let (tx, rx) = unbounded();

    // Start Command Thread
    {
        let m = matche.clone();
        let tx = tx.clone();
        let _ = thread::spawn(move || loop {
            // Create cmd..
            let mut exe = exec::ExecuteCommand::new(tx.clone());

            // Set shell command
            exe.shell_command = m.value_of("shell_command").unwrap().to_string();

            // Set command
            exe.command = m.values_of_lossy("command").unwrap();

            // Set is exec flag.
            exe.is_exec = is_exec;

            // Exec command
            exe.exec_command();

            // sleep interval
            std::thread::sleep(Duration::from_secs_f64(interval));
        });
    }

    // check batch mode
    if !batch {
        // is watch mode
        // Create view
        let mut view = view::View::new()
            // Set interval on view.header
            .set_interval(interval)
            // Set color in view
            .set_color(color)
            // Set line number in view
            .set_line_number(line_number)
            // Set diff(watch diff) in view
            .set_watch_diff(diff);

        // Set logfile
        if logfile != None {
            view = view.set_logfile(logfile.unwrap().to_string());
        }

        // start app.
        let _res = view.start(tx.clone(), rx);
    } else {
        // is batch mode
        println!("is batch (developing now)");
    }
}
