extern crate ncurses;

pub mod watch;

use std::sync::mpsc::{Receiver,Sender};

use event::Event;

pub struct View {
    pub done: bool,
    pub watch: watch::Watch,
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}


impl View {
    pub fn new(watch: watch::Watch,
               tx: Sender<Event>,
               rx: Receiver<Event>) -> Self {
        Self {
            done: false,
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

    pub fn run(&mut self){
        while !self.done {
            match self.rx.try_recv(){
                Ok(Event::OutputUpdate(_cmd)) => self.watch.update(_cmd),
                Ok(Event::Exit) => self.done = true,
                Ok(Event::Input(i)) => {
                    match i {
                        ncurses::KEY_UP => self.watch.scroll_up(),
                        ncurses::KEY_DOWN => self.watch.scroll_down(),
                        // escape key(0x1b),q(0x71)
                        ncurses::KEY_F1 | 0x1b | 0x71 => self.exit(),
                        _ => {}
                    }
                }
             _ => {}
            };
        }
    }
}