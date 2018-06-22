#[macro_use]
extern crate clap;
extern crate ncurses;

mod cmd;
mod common;
mod event;
mod input;
mod view;

use self::clap::{App, Arg, AppSettings};
use std::sync::mpsc::{channel, Receiver};
use std::env::args;
use std::time::Duration;
use std::thread;

use event::Event;
use input::Input;

struct Main {
    done: bool,
    view: view::watch::Watch,
    rx: Receiver<Event>,
}

impl Main {
    fn init(&mut self){
        self.view.init();
    }

    fn main(&mut self){
        while !self.done {
            match self.rx.try_recv(){
                Ok(Event::OutputUpdate(_cmd)) => self.view.update(_cmd),
                Ok(Event::Exit) => self.done = true,
                Ok(Event::Input(i)) => {
                    match i {
                        ncurses::KEY_UP => self.view.scroll_up(),
                        ncurses::KEY_DOWN => self.view.scroll_down(),
                        ncurses::KEY_F1 => self.view.exit(),
                        _ => {}
                    }
                }
             _ => {}
            };
        }
    }
}



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
    let mut _view = view::watch::Watch::new();

    // create channel
    let (tx, rx) = channel();
    let mut _m = Main {
        done: false,
        view: _view,
        rx: rx
    };
    _m.init();

    // Create input
    let mut _input = Input::new(tx.clone());

    // Start Command Thread
    {
        let tx = tx.clone();
        
        thread::spawn(move || loop {
            let mut command = cmd::CmdRun::new(tx.clone());
            command.cmd.command = _matches.values_of_lossy("command").unwrap().join(" ");
            command.exec();

            // Get now time
            let _now = common::now_str();

            // set command struct
            let cmd = cmd::Cmd {
                timestamp: command.cmd.timestamp,
                command: command.cmd.command,
                status: command.cmd.status,
                stdout: command.cmd.stdout,
                stderr: command.cmd.stderr
            };
            
            // send output update
            let _ = tx.send(Event::OutputUpdate(cmd));

            // sleep interval
            thread::sleep(Duration::from_secs(_interval));
        });
    }
    _input.run();
    _m.main();
}