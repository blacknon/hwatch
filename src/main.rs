// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.8
// TODO(blacknon): Number, Color, Beepなどの表示を工夫する(true時は太字で色付き、false時は色を薄くする、みたいな感じで…)
// TODO(blacknon): diffのある箇所だけを表示するモードの作成.
//                 `Line(Only)`, `Word(Only)` mode.
//                 フラグにして、Line/Word diff時のみ有効にするような変更とする.
// TODO(blacknon): 出力結果が変わった場合やコマンドの実行に失敗・成功した場合に、オプションで指定したコマンドをキックする機能を追加.
//                 - その際、環境変数をキックするコマンドに渡して実行結果や差分をキック先コマンドで扱えるようにする。
//                 - また、実行時にはシェルも指定して呼び出せるようにする？

// v0.3.9
// TODO(blacknon): コマンド実行結果のみを表示するオプション(keybind)の追加(なんかもうコードあるっぽい？？).
//                 - https://github.com/blacknon/hwatch/issues/63
// TODO(blacknon): セキュリティのため、heaplessのバージョンを上げる
// TODO(blakcnon): batch modeの実装.
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能にする
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man
// TODO(blacknon): errorとの比較を行わない(正常終了時のみを比較対象とし、errorの履歴をスキップしてdiffする)キーバインドの追加(なんかのmode?)
// TODO(blacknon): ライフタイムの名称をちゃんと命名する。
// TODO(blacknon): エラーなどのメッセージ表示領域の作成
// TODO(blacknon): diffのライブラリをsimilarに切り替える？
//                 - https://github.com/mitsuhiko/similar

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
extern crate question;

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
pub const SHELL_COMMAND_EXECCMD: &str = "{COMMAND}";

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
        // Option to specify the command to be executed when the output fluctuates.
        //     [-C,--changed-command]
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
        // Enable line number mode option
        //   [--line-number,-N]
        .arg(
            Arg::new("line_number")
                .help("show line number")
                .short('N')
                .long("line-number"),
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

fn main() {
    // Get command args matches
    let matche = build_app().get_matches();

    // Get options flag
    // let batch = matche.is_present("batch");
    let diff = matche.is_present("differences");
    let beep = matche.is_present("beep");
    let color = matche.is_present("color");
    let is_exec = matche.is_present("exec");
    let line_number = matche.is_present("line_number");

    // Get options value
    // let interval: f64 = value_t!(matche, "interval", f64).unwrap_or_else(|e| e.exit());
    let interval: f64 = matche.value_of_t_or_exit("interval");

    // let exec = matche.value_of("exec");
    let logfile = matche.value_of("logfile");

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
            println!("file {:?} is exists.", _abs_log_path);
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
            println!("directory {:?} is not exists.", _abs_log_dir);
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
    // if !batch {
    // is watch mode
    // Create view
    let mut view = view::View::new()
        // Set interval on view.header
        .set_interval(interval)
        .set_beep(beep)
        // Set color in view
        .set_color(color)
        // Set line number in view
        .set_line_number(line_number)
        // Set diff(watch diff) in view
        .set_watch_diff(diff);

    // Set logfile
    if let Some(logfile) = logfile {
        view = view.set_logfile(logfile.to_string());
    }

    // start app.
    let _res = view.start(tx, rx);
    // } else {
    //     // is batch mode
    //     println!("is batch (developing now)");
    // }
}
