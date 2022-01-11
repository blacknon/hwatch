// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: input signalしか無いなら、view配下に移動する
// TODO: signalをOSごとにuseを切り替える(Windows対応)

use exec::Result;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, SIGINT}; // Linux
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

static mut INPUT_SIGNAL: i32 = 0;
extern "C" fn signal_handler(signum: i32) {
    unsafe { INPUT_SIGNAL = signum }
}

pub enum AppEvent {
    OutputUpdate(Result),
    Input(i32),
    Signal(i32),
    Exit,
}

pub struct Signal {
    tx: Sender<AppEvent>,
}

/// Signal Trait
impl Signal {
    pub fn new(tx: Sender<AppEvent>) -> Self {
        //! new signal
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
                let _ = self.tx.send(AppEvent::Signal(INPUT_SIGNAL));
                drop(INPUT_SIGNAL);
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
