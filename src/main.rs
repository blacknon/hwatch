// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.1
// TODO(blacknon): 行頭に行番号を表示する機能の追加.(v0.3.1)
//                 `n`キーでの切り替えが良いか? diffでの出力をどうするかがポイントかも？？

// v0.3.2
// TODO(blakcnon): batch modeの実装(v0.3.2).
// TODO(blacknon): コマンドがエラーになった場合はそこで終了する機能の追加(v0.3.2)
//                 watchコマンドにもある(-e, --errexit)
// TODO(blacknon): 出力結果が変わった場合はそこで終了する機能の追加(v0.3.2)
//                 watchコマンドにもある(-g, --chgexit)
// TODO(blacknon): 出力結果が変わった場合はbeepを鳴らす機能の追加(v0.3.2)
//                 watchコマンドにもある(-b, --beep)。微妙に機能としては違うものかも…？
// TODO(blacknon): 出力結果が変わった場合やコマンドの実行に失敗・成功した場合に、オプションで指定したコマンドをキックする機能を追加. (v0.3.2)
//                 その際、環境変数をキックするコマンドに渡して実行結果や差分をキック先コマンドで扱えるようにする。

// v0.3.3
// TODO(blacknon): Windows対応(v0.3.3). 一応、あとはライブラリが対応すればイケる.
// TODO(blacknon): 任意時点間のdiffが行えるようにする(v0.3.3).
// TODO(blacknon): diffのある箇所だけを表示するモードの作成(v0.3.3).
//                 `OnlyLine`, `OnlyWord` mode.
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する(v0.3.3)
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく(v0.3.3)
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる (v0.3.3)
//                 https://github.com/rust-cli/man

#[warn(unused_doc_comments)]
// crate
extern crate ansi4tui;
extern crate ansi_parser;
extern crate chrono;
extern crate crossterm;
extern crate difference;
extern crate heapless;
extern crate regex;
extern crate serde;
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
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

// local modules
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

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

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
        // -- options --
        // Enable batch mode option
        //     [-b,--batch]
        // .arg(
        //     Arg::with_name("batch")
        //         .help("output exection results to stdout")
        //         .short("b")
        //         .long("batch"),
        // )
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
        // @TODO: v1.0.0
        //        通常のwatchでも、-xはフラグとして扱われている可能性が高い。
        //        なので、こちらでも引数を取るような方式ではなく、フラグとして扱ったほうがいいだろう。
        // exec
        // .arg(
        //     Arg::with_name("exec")
        //         .help("pass command to exec instead of 'sh -c'")
        //         .short("x")
        //         .long("exec")
        //         .takes_value(true)
        //         .default_value("sh -c"),
        // )
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
    let _matches = build_app().get_matches();

    // matches clone
    let _m = _matches.clone();

    // Get options flag
    let batch = _m.is_present("batch");
    let diff = _m.is_present("differences");
    let color = _m.is_present("color");
    let line_number = _m.is_present("line_number");

    // Get options value
    let interval: f64 = value_t!(_matches, "interval", f64).unwrap_or_else(|e| e.exit());
    // let exec = _m.value_of("exec");
    let logfile = _m.value_of("logfile");

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
    let (tx, rx) = channel();

    // Start Command Thread
    {
        let tx = tx.clone();
        let _ = thread::spawn(move || loop {
            // Create cmd..
            let mut exe = exec::ExecuteCommand::new(tx.clone());

            // Set command
            exe.command = _matches.values_of_lossy("command").unwrap().join(" ");

            // Exec command
            exe.exec_command();

            // sleep interval
            thread::sleep(Duration::from_secs_f64(interval));
        });
    }

    // check batch mode
    if !batch {
        // is watch mode
        // Create view
        let mut view = view::View::new();

        // Set interval on view.header
        view.set_interval(interval);

        // Set color in view
        view.set_color(color);

        // Set color in view
        view.set_line_number(line_number);

        // Set diff(watch diff) in view
        view.set_watch_diff(diff);

        // Set logfile
        if logfile != None {
            view.set_logfile(logfile.unwrap().to_string());
        }

        // start app
        let _res = view.start(tx.clone(), rx);
    } else {
        // is batch mode
        println!("is batch (developing now)");
    }
}
