// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// macro crate
#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

extern crate itertools;
extern crate ncurses;
extern crate nix;
extern crate regex;

// modules
use clap::{App, AppSettings, Arg};
use std::env::args;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

// local modules
mod cmd;
mod common;
mod event;
mod input;
mod signal;
mod view;
use input::Input;
use signal::Signal;
use view::View;

// const
pub const HISTORY_WIDTH: i32 = 21;
pub const IS_WATCH_PAD: i32 = 0;
pub const IS_HISTORY_PAD: i32 = 1;
pub const IS_STDOUT: i32 = 1;
pub const IS_STDERR: i32 = 2;
pub const IS_OUTPUT: i32 = 3;
pub const DIFF_DISABLE: i32 = 0;
pub const DIFF_WATCH: i32 = 1;
pub const DIFF_LINE: i32 = 2;

// Parse args and options
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
        .arg(
            Arg::with_name("batch")
                .help("output exection results to stdout")
                .short("b")
                .long("batch"),
        )
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
        //   [--logging,-l] /path/to/logfile
        // TODO(blacknon): jsonで出力させる。outputはBase64変換して保持
        .arg(
            Arg::with_name("log")
                .help("logging file")
                .short("l")
                .long("logfile"),
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
        //
        // Interval optionMacの場合は更に背景画像も変わる
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
    // Get command args
    let _matches = build_app().get_matches();

    // Get options
    let mut _interval: u64 = _matches
        .value_of("interval")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let mut _batch = _matches.is_present("batch");
    let mut _diff = _matches.is_present("differences");
    let mut _color = _matches.is_present("color");
    let mut _exec = _matches.values_of_lossy("exec");

    // Create channel
    let (tx, rx) = channel();

    // Start Command Thread
    {
        let tx = tx.clone();
        let _ = thread::spawn(move || loop {
            let mut cmd = cmd::CmdRun::new(tx.clone());

            // TODO(blacknon): cmd.intervalいらないんじゃね？後で消す
            // cmd.interval = _interval.clone();
            cmd.command = _matches.values_of_lossy("command").unwrap().join(" ");
            cmd.exec_command();

            // Get now time
            let _now = common::now_str();

            // sleep interval
            thread::sleep(Duration::from_secs(_interval));
        });
    }

    // check batch mode
    if !_batch {
        // is not batch mode

        // Create view
        let mut _view = View::new(tx.clone(), rx, _diff, _color);

        //

        // Create input
        let mut _input = Input::new(tx.clone());

        // Create signal
        let mut _signal = Signal::new(tx.clone());

        // await input thread
        _input.run();

        // await signal thread
        _signal.run();

        // view
        _view.get_event();
    } else {
        // is batch mode
        print!("is batch\n");
    }
}
