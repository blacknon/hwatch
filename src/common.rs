// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use chrono::Local;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::prelude::*;

// local module
use crate::exec::CommandResult;

pub fn now_str() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
}

/// logging result data to log file(_logpath).
pub fn logging_result(_logpath: &str, _result: &CommandResult) -> Result<(), Box<dyn Error>> {
    // try open logfile
    let mut logfile = match OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(_logpath)
    {
        Err(why) => return Err(Box::new(why)),
        Ok(file) => file,
    };

    // create logline
    let logdata = serde_json::to_string(&_result)?;

    // write log
    // TODO(blacknon): warning出てるので対応
    _ = writeln!(logfile, "{logdata}");

    Ok(())
}
