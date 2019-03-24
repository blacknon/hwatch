// macro crate
#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

// crate
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
        // command
        .arg(
            Arg::with_name("command")
                .allow_hyphen_values(true)
                .multiple(true)
                .required(true),
        )
        // options
        // .arg(Arg::with_name("color")
        //     .help("interpret ANSI color and style sequences")
        //     .short("c")
        //     .long("color")
        // )
        // Enable diff mode option
        //   --differences,-d
        .arg(
            Arg::with_name("differences")
                .help("highlight changes between updates")
                .short("d")
                .long("differences"),
        )
        // .arg(Arg::with_name("exec")
        //     .help("pass command to exec instead of 'sh -c'")
        //     .short("x")
        //     .long("exec")
        //     .takes_value(true)
        //     .default_value("sh -c")
        // )
        // Interval option
        //   --interval,-n
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
    let mut _diff = _matches.is_present("differences");

    // Create channel
    let (tx, rx) = channel();

    // Create view
    let mut _view = View::new(tx.clone(), rx, _diff);

    // Create input
    let mut _input = Input::new(tx.clone());

    // Create signal
    let mut _signal = Signal::new(tx.clone());

    // Start Command Thread
    {
        let tx = tx.clone();
        let _ = thread::spawn(move || loop {
            let mut cmd = cmd::CmdRun::new(tx.clone());
            cmd.interval = _interval.clone();
            cmd.command = _matches.values_of_lossy("command").unwrap().join(" ");
            cmd.exec_command();

            // Get now time
            let _now = common::now_str();

            // sleep interval
            thread::sleep(Duration::from_secs(_interval));
        });
    }

    // await input thread
    _input.run();

    // await signal thread
    _signal.run();

    // view
    _view.get_event();
}
