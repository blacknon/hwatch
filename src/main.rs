// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): マニュアル(manのデータ)を作成 (v0.2.0)
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する(v1.0.0)
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): コマンドがエラーになった場合はそこで終了する機能の追加(v1.0.0)
//                 watchコマンドにもある(-e, --errexit)
// TODO(blacknon): 出力結果が変わった場合はそこで終了する機能の追加(v1.0.0)
//                 watchコマンドにもある(-g, --chgexit)
// TODO(blacknon): 出力結果が変わった場合はbeepを鳴らす機能の追加(v1.0.0)
//                 watchコマンドにもある(-b, --beep)。微妙に機能としては違うものかも…？
// TODO(blacknon): 検索によるフィルタリング機能の追加.(v0.2.1)
//                 (`/`キーで処理。正規表現検索も機能としてデフォルトで有効にしたいが、果たして…？ できればリアルタイムフィルタリングを行いたいところだけど…？)
// TODO(blacknon): 行頭に行番号を表示する機能の追加.(v0.2.1)
//                 `n`キーでの切り替えが良いか❓ diffでの出力をどうするかがポイントかも？？
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく(v0.2.0)
// TODO(blacknon): Resultのメモリ解放できてないっぽい(全部溜め込んでるっぽい)ので、対処する
// TODO(blacknon): 長いcommand指定時は省略して出力させる
// TODO(blacknon): ncursesからtui-rsを利用した方式に切り替える

// TODO(blacknon): メンテナンスの都合から、colorとdiffは同時には使えないようにしよう(diffが勝つ方式に変更. あとからcolorを有効にした場合はcolorを有効化してdiff無効化にする)

#[warn(unused_doc_comments)]
// crate
extern crate ansi4tui;
extern crate chrono;
extern crate crossterm;
extern crate difference;
extern crate regex;
extern crate serde;
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
mod common;
mod diff;
mod event;
mod exec;
mod header;
mod history;
mod view;
mod watch;

// const
pub const DEFAULT_INTERVAL: f64 = 2.0;
pub const HISTORY_WIDTH: u16 = 25;

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
    let mut _batch = _m.is_present("batch");
    let mut _diff = _m.is_present("differences");
    let mut _color = _m.is_present("color");

    // Get options value
    let mut _interval: f64 = value_t!(_matches, "interval", f64).unwrap_or_else(|e| e.exit());
    let mut _exec = _m.value_of("exec");
    let mut _logfile = _m.value_of("logfile");

    // check _logfile directory
    // TODO(blacknon): commonに移す？(ここで直書きする必要性はなさそう)
    if _logfile != None {
        let _log_path = Path::new(_logfile.clone().unwrap());
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
            thread::sleep(Duration::from_secs_f64(_interval));
        });
    }

    // check batch mode
    if !_batch {
        // is watch mode
        // Create view
        let mut _view = view::View::new();

        // Set interval on _view.header
        _view.set_interval(_interval);

        // Set logfile
        // if _logfile != None {
        //     _view.set_logfile(_logfile.unwrap().to_string());
        // }

        // Set diff in _view
        // let mut _diff_type = 0;
        // if _diff {
        //     _diff_type = 1;
        // }
        // _view.switch_diff(_diff_type);

        // Set color in _view
        // _view.set_color(_color);

        // Create input
        // let mut _input = Input::new(tx.clone());

        // Create signal
        // let mut _signal = Signal::new(tx.clone());

        // await input thread
        // _input.run();

        // await signal thread
        // _signal.run();

        // view
        // _view.get_event();

        // start app
        let _res = _view.start(tx.clone(), rx);
    } else {
        // is batch mode
        println!("is batch (developing now)");
    }
}
