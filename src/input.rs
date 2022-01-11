// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: signalに移動(多分分けなくてもいい気がする)

use event::Event;
use ncurses::*;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

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
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
