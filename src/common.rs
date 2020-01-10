// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// crate
extern crate chrono;

// module
use self::chrono::Local;
use std::fs::OpenOptions;
use std::io::prelude::*;

// local module
use cmd::Result;

//
pub fn now_str() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S").to_string();
}

// logging result data to log file(_logpath).
pub fn logging_result(_logpath: &String, _result: &Result) -> serde_json::Result<()> {
    // Open logfile
    let mut logfile = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(_logpath)
        .unwrap();

    // create logline
    let logdata = serde_json::to_string(&_result)?;

    // write log
    writeln!(logfile, "{}", logdata);

    Ok(())
}
