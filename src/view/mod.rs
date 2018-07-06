extern "C" {
    pub fn setlocale(category: i32, locale: *const u8) -> *const u8;
}
extern crate ncurses;

pub mod watch;
mod diff;
mod history;


use std::sync::mpsc::{Receiver,Sender};

use ncurses::*;

use cmd::Result;
use event::Event;

pub struct View {
    pub done: bool,
    pub diff: bool,

    pub screen: ncurses::WINDOW,
    pub history: history::History,
    pub watch: watch::Watch,

    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}


impl View {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        unsafe {
            setlocale(0 /* = LC_CTYPE */, "".as_ptr());
        }
        let _screen = initscr();
        start_color();
        use_default_colors();
        cbreak();
        keypad(_screen, true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);       

        // set color
        init_pair(1, -1, -1); // fg=default, bg=clear
        init_pair(2, COLOR_GREEN, -1); // fg=green, bg=clear
        init_pair(3, COLOR_RED, -1); // fg=red, bg=clear
        init_pair(11, COLOR_WHITE, COLOR_RED); // fg=white, bg=red

        Self {
            done: false,
            diff: true,

            screen: _screen,
            history: history::History::new(_screen.clone()),
            watch: watch::Watch::new(_screen.clone()),

            tx: tx,
            rx: rx
        }
    }

    fn exit(&mut self){
        self.watch.exit();
        self.history.exit();
        let _ = self.tx.send(Event::Exit);
    }

    fn output_update(&mut self, _result: Result) {
        if self.history.get_latest_history().output != _result.output {
            // update diff watch screen
            let _history_count = self.history.count.clone();

            // watch diff mode
            if self.diff && _history_count > 0 {
                self.watch.result = _result.clone();
                self.watch.before_update_output_pad();
                diff::watch_diff(
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
            self.history.draw_history_pad();
        } else {
            // update watch screen
            self.watch.update(_result.clone());
        }
    }

    // fn show_history(&mut self) {
    //     if self.history_mode == false {
    //         clear();
    //         self.history_mode = true;
    //         self.history.start_pad();
    //         overlay(self.history.history_pad, self.watch.pad);
    //     } else {
    //         clear();
    //         self.history.exit_pad();
    //         self.history_mode = false;
    //         self.watch.draw_output_pad();
    //         // overlay(self.watch.pad,self.history.history_pad);
    //     }
    // }

    // start input reception
    pub fn start_reception(&mut self){
        while !self.done {
            match self.rx.try_recv(){
                Ok(Event::OutputUpdate(_cmd)) => self.output_update(_cmd),
                Ok(Event::Exit) => self.done = true,
                Ok(Event::Input(i)) => {
                    match i {
                        // Screen Resize
                        ncurses::KEY_RESIZE => self.watch.resize(),

                        // watch pad up/down
                        ncurses::KEY_UP => self.watch.scroll_up(),
                        ncurses::KEY_DOWN => self.watch.scroll_down(),

                        // Shift + Up
                        // ncurses::KEY_SR => self.history.scroll_up(),

                        // Shift + Down
                        // ncurses::KEY_SF => self.history.scroll_down(),

                        // ESC(0x1b),q(0x71)
                        ncurses::KEY_F1 | 0x1b | 0x71 => self.exit(),
                        // h(0x68)
                        // 0x68 => self.show_history(),
                        _ => {}
                    }
                }
             _ => {}
            };
        }
    }
}
