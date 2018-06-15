#[macro_use]
extern crate clap;

use self::clap::{App, Arg, AppSettings};
use std::env::args;

mod cmd;
mod common;


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
        .arg(Arg::from_usage("-i --interval=[secs]  'seconds to wait between updates'"))
        .arg(Arg::from_usage("-x --exec=[exec]  'pass command to exec instead of \"sh -c\"'"))
}


fn main() {
    // get command args
    let _matches = build_app().get_matches();

    // set command args to var
    let _commands = _matches.values_of_lossy("command").unwrap().join(" ");

    // get now time string ("yyyy/mm/dd HH:MM:SS")
    let now = common::now_str();
    println!("{:}",now);

    // set command infomation
    let mut command = cmd::Cmd {
        command: _commands,
        status: false,
        stdout: "".to_string(),
        stderr: "".to_string() 
    };

    // run command
    command.run();

    println!("{:?}", command.status);

    if command.stdout.len() > 0{
        print!("stdout:\n{}", command.stdout);
    }

    if command.stderr.len() > 0{
        println!("stderr:\n{}", command.stderr);
    }
}