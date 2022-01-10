// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use std::sync::mpsc::{Receiver, Sender};
use termion;

// local module
use event::Event;

/// Struct at watch view window.
pub struct View {
    pub done: bool,
    // pub header: header::Header,
    pub header: bool,
    // pub watch: watch::Watch,
    pub watch: bool,
    // pub history: history::History,
    pub history: bool,
    pub logfile: String,
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}

/// Trail at watch view window.
impl View {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        //! method at create new view trail.

        Self {
            done: false,
            header: true,
            watch: true,
            history: true,
            logfile: "".to_string(),
            tx: tx,
            rx: rx,
        }
    }
}
