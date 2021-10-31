// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use event::Event;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, SIGINT};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

static mut INPUT_SIGNAL: i32 = 0;

extern "C" fn signal_handler(signum: i32) {
    unsafe { INPUT_SIGNAL = signum }
}

pub struct Signal {
    tx: Sender<Event>,
}

impl Signal {
    pub fn new(tx: Sender<Event>) -> Self {
        Signal { tx: tx }
    }

    pub fn run(self) {
        let sa = SigAction::new(
            SigHandler::Handler(signal_handler),
            SaFlags::SA_NODEFER,
            SigSet::empty(),
        );
        unsafe { sigaction(SIGINT, &sa) }.unwrap();

        let _ = thread::spawn(move || unsafe {
            loop {
                let _ = self.tx.send(Event::Signal(INPUT_SIGNAL));
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
