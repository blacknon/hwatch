// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// crate
extern crate chrono;

// module
use self::chrono::Local;

pub fn now_str() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S").to_string();
}

