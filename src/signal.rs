use nix::sys::signal;
use nix::sys::signal::{sigaction, SigAction, SigHandler, SaFlags, SigSet};
// use std::process;

use std::thread;
use event::Event;
use std::sync::mpsc::Sender;
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
        unsafe { sigaction(signal::SIGINT, &sa) }.unwrap();

        let _ = thread::spawn(move || unsafe {
            // let signal = INPUT_SIGNAL;
            loop {
                let _ = self.tx.send(Event::Signal(INPUT_SIGNAL));

                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
