// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

// local module
use cmd::Result;
use event::Event;

pub struct Batch {
    pub done: bool,
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}

impl Batch {
    pub fn new(&mut self, tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        Self {
            done: false,
            tx: tx,
            rx: rx,
        }
    }

    pub fn get_event(&mut self) {
        while !self.done {
            match self.rx.try_recv() {
                Ok(Event::OutputUpdate(_cmd)) => self.update(_cmd),
                Ok(Event::Exit) => self.done = true,
            };
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub fn update(&mut self, _result: Result) {}
}
