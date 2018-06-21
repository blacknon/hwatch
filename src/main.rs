#[macro_use]
extern crate clap;

use self::clap::{App, Arg, AppSettings};
use std::env::args;
use std::time::Duration;
use std::thread;


mod cmd;
mod common;
mod ncurse;


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

        // command
        .arg(Arg::with_name("command")
            .multiple(true)
            .required(true)
        )

        // options
        .arg(Arg::with_name("interval")              
            .help("seconds to wait between updates")
            .short("i")
            .long("interval")
            .takes_value(true)
            .default_value("10")
        )
        // .arg(Arg::with_name("exec")              
        //     .help("pass command to exec instead of 'sh -c'")
        //     .short("x")
        //     .long("exec")
        //     .takes_value(true)
        //     .default_value("sh -c")
        // )
}


fn main() {
    // get command args
    let _matches = build_app().get_matches();
    let mut _view = ncurse::View::new();

    // get interval secs
    let mut _interval:u64 = _matches.value_of("interval").unwrap().parse::<u64>().unwrap();

    loop {
        // run command
        let mut command = cmd::Cmd::new();
        command.command = _matches.values_of_lossy("command").unwrap().join(" ");
        command.run();

        // get now time string ("yyyy/mm/dd HH:MM:SS")
        let _now = common::now_str();

        // Setup view
        _view.timestamp = _now;
        _view.command = command.command;
        _view.stdout = command.stdout;
        _view.stderr = command.stderr;
        _view.status = command.status;

        // view screen
        _view.view_watch_screen();

        // sleep time(interval)
        thread::sleep(Duration::from_secs(_interval));
    }
}