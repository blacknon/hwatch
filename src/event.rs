// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crate::exec::CommandResult;
use crate::app::ResultItems;

pub enum AppEvent {
    OutputUpdate(CommandResult),
    HistoryUpdate((ResultItems, ResultItems, ResultItems), bool),
    TerminalEvent(crossterm::event::Event),
    Redraw,
    ChangeFlagMouseEvent,
    Exit,
}
