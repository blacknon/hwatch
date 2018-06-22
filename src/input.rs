use ncurses::*;
use std::thread;
use event::Event;
use std::sync::mpsc::Sender;

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
                                  while ch != 0x71 {
                                      let _ = self.tx.send(Event::Input(ch));
                                      ch = getch();
                                  }
                                  let _ = self.tx.send(Event::Exit);
                              });
    }
}
