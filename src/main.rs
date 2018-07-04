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

use input::Input;
use view::View;


// Parse args and options
fn build_app() -> clap::App<'static, 'static> {
    // get own name
    let _program = args()
                    .nth(0)
                    .and_then(
                        |s| {
                            std::path::PathBuf::from(s)
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                        }
                    )
                    .unwrap();

    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::AllowLeadingHyphen)

        // command
        .arg(Arg::with_name("command")
            .allow_hyphen_values(true)
            .multiple(true)
            .required(true)
        )

        // options
        .arg(Arg::with_name("color")
            .help("interpret ANSI color and style sequences")
            .short("c")
            .long("color")
        )
        .arg(Arg::with_name("differences")
            .help("highlight changes between updates")
            .short("d")
            .long("differences")
        )
        .arg(Arg::with_name("interval")
            .help("seconds to wait between updates")
            .short("n")
            .long("interval")
            .takes_value(true)
            .default_value("2")
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

    // get options
    let mut _interval:u64 = _matches.value_of("interval").unwrap().parse::<u64>().unwrap();
    let mut _diff = _matches.is_present("differences");

    // create channel
    let (tx, rx) = channel();

    // create view
    let mut _view = View::new(tx.clone(), rx);
    _view.diff = _diff;
    _view.init();

    // Create input
    let mut _input = Input::new(tx.clone());


    // Start Command Thread
    {
        let tx = tx.clone();
        let _ = thread::spawn(move ||
            loop {
                let mut cmd = cmd::CmdRun::new(tx.clone());
                cmd.command = _matches.values_of_lossy("command").unwrap().join(" ");
                cmd.exec_command();

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
    _view.start_reception();
}