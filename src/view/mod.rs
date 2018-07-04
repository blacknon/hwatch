extern crate ncurses;

pub mod watch;
mod diff;

use std::sync::mpsc::{Receiver,Sender};

use cmd::Result;
use self::diff as self_diff;
use event::Event;
use history::History;

pub struct View {
    pub done: bool,
    pub diff: bool,
    pub history: History,
    pub history_mode: bool,
    pub watch: watch::Watch,
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}


impl View {
    pub fn new(watch: watch::Watch,
               history: History,
               tx: Sender<Event>,
               rx: Receiver<Event>) -> Self {
        Self {
            done: false,
            diff: true,
            history: history,
            history_mode: false,
            watch: watch,
            tx: tx,
            rx: rx
        }
    }

    pub fn init(&mut self){
        self.watch.init();
    }

    fn exit(&mut self){
        self.watch.exit();
        let _ = self.tx.send(Event::Exit);
    }

    fn output_update(&mut self, _result: Result) {
        if self.history.get_latest_history().output != _result.output {
            // update diff watch screen
            let _history_count = self.history.count;
            if self.diff && _history_count > 0 {
                self.watch.result = _result.clone();
                self.watch.before_update_output_pad();
                self_diff::watch_diff(
                    self.watch.clone(),
                    self.history.get_latest_history().output.clone(),
                    _result.output.clone()
                );
                self.watch.draw_output_pad();
            } else {
                self.watch.update(_result.clone());
            }

            // append history
            self.history.append_history(_result.clone());
        } else {
            // update watch screen
            self.watch.update(_result.clone());
        }
    }

    // start input reception
    pub fn start_reception(&mut self){
        while !self.done {
            match self.rx.try_recv(){
                Ok(Event::OutputUpdate(_cmd)) => self.output_update(_cmd),
                Ok(Event::Exit) => self.done = true,
                Ok(Event::Input(i)) => {
                    match i {
                        ncurses::KEY_UP => self.watch.scroll_up(),
                        ncurses::KEY_DOWN => self.watch.scroll_down(),
                        // ESC(0x1b),q(0x71)
                        ncurses::KEY_F1 | 0x1b | 0x71 => self.exit(),
                        _ => {}
                    }
                }
             _ => {}
            };
        }
    }
}
