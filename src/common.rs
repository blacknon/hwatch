// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use chrono::Local;
use serde_json;
use std::fs::OpenOptions;
use std::io::prelude::*;

// local module
use exec::CommandResult;

pub fn now_str() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
}

/// logging result data to log file(_logpath).
pub fn logging_result(_logpath: &String, _result: &CommandResult) -> serde_json::Result<()> {
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
    // TODO(blacknon): warning出てるので対応
    writeln!(logfile, "{}", logdata);

    Ok(())
}

///
pub fn differences_result(_result1: &CommandResult, _result2: &CommandResult) -> bool {
    // result
    let mut result = true;

    // command
    if _result1.command != _result2.command {
        result = false;
    }

    // status
    if _result1.status != _result2.status {
        result = false;
    }

    // output
    if _result1.output != _result2.output {
        result = false;
    }

    // stdout
    if _result1.stdout != _result2.stdout {
        result = false;
    }

    // stderr
    if _result1.stderr != _result2.stderr {
        result = false;
    }

    return result;
}
