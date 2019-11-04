// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use cmd::Result;

pub enum Event {
    OutputUpdate(Result),
    Input(i32),
    Signal(i32),
    Exit,
}
