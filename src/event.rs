// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crate::exec::CommandResult;

pub enum AppEvent {
    OutputUpdate(CommandResult),
    TerminalEvent(crossterm::event::Event),
    Exit,
}
