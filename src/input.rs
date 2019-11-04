// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use event::Event;
use ncurses::*;
use std::sync::mpsc::Sender;
use std::thread;

pub struct Input {
    tx: Sender<Event>,
}

impl Input {
    pub fn new(tx: Sender<Event>) -> Self {
        Input { tx: tx }
    }

    pub fn run(self) {
        let _ = thread::spawn(move || {
            let mut ch = getch();
            loop {
                let _ = self.tx.send(Event::Input(ch));
                ch = getch();
            }
        });
    }
}
