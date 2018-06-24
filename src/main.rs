#[macro_use]
extern crate clap;
extern crate ncurses;

mod cmd;
mod common;
mod event;
mod input;
mod view;

use std::sync::mpsc::channel;
use std::env::args;
use std::time::Duration;
use std::thread;

use self::clap::{App, Arg, AppSettings};

use view::View;
use input::Input;


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

    // get interval secs
    let mut _interval:u64 = _matches.value_of("interval").unwrap().parse::<u64>().unwrap();

    // start view
    let mut _watch = view::watch::Watch::new();

    // create channel
    let (tx, rx) = channel();
    let mut _view = View::new(_watch,tx.clone(),rx);
    _view.init();

    // Create input
    let mut _input = Input::new(tx.clone());

    // Start Command Thread
    let mut command = cmd::CmdRun::new(tx.clone());
    command.command = _matches.values_of_lossy("command").unwrap().join(" ");
    {
        let tx = tx.clone();
        thread::spawn(move ||
            loop {
                let mut command = cmd::CmdRun::new(tx.clone());
                command.command = _matches.values_of_lossy("command").unwrap().join(" ");
                command.exec_command();

                // Get now time
                let _now = common::now_str();

                // sleep interval
                thread::sleep(Duration::from_secs(_interval));
            }
        );
    }

    // await input thread
    _input.run();

    // view
    _view.run();
}