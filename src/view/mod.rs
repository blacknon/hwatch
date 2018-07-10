extern "C" {
    pub fn setlocale(category: i32, locale: *const u8) -> *const u8;
}

mod header;
mod watch;


use std::sync::mpsc::{Receiver,Sender};

use ncurses::*;

use cmd::Result;
use event::Event;
use self::watch::Watch;

pub struct View {
    pub done: bool,

    pub screen: WINDOW,
    pub header: header::Header,
    pub watch: watch::Watch,

    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}


impl View {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>, _diff: bool) -> Self {
        // Create ncurses screen
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
        init_pair(11, COLOR_BLACK, COLOR_WHITE); // fg=black, bg=white
        init_pair(12, COLOR_WHITE, COLOR_RED); // fg=white, bg=red
        init_pair(13, COLOR_WHITE, COLOR_GREEN); // fg=white, bg=green
        
        let _watch = Watch::new(_screen.clone(), _diff);
        Self {
            done: false,

            screen: _screen,
            header: header::Header::new(_screen.clone()),
            watch: _watch,

            tx: tx,
            rx: rx
        }
    }

    fn exit(&mut self) {
        self.watch.exit();
        let _ = self.tx.send(Event::Exit);
    }

    fn output_update(&mut self, _result: Result) {
        // update header
        self.header.update_header(_result.clone());

        // update before result
        let before_result = self.watch.latest_result.clone();
        self.watch.before_result = before_result;

        // update latest result
        self.watch.latest_result = _result.clone();

        // history append result
        if self.watch.get_latest_history().output != _result.output {
            clear();
            self.header.update_header(_result.clone());
            self.watch.append_history(_result.clone());

            // add selected positon
            if self.watch.selected_position != 0 {
                self.watch.selected_position += 1;
            }
            self.watch.draw_history_pad();
            self.watch.watch_update();
        }

        // if history selected latest, update watch window.
        if self.watch.selected_position == 0 {
            self.header.update_header(_result.clone());
            self.watch.draw_history_pad();
            self.watch.watch_update();
        }
    }

    fn history_scroll_up(&mut self) {
        if self.watch.selected_position > 0 {
            let latest_result = self.watch.latest_result.clone();
            clear();
            self.header.update_header(latest_result);
            self.watch.history_scroll_up()
        }
    }

    fn history_scroll_down(&mut self) {
        if self.watch.count > self.watch.selected_position {
            let latest_result = self.watch.latest_result.clone();
            clear();
            self.header.update_header(latest_result);
            self.watch.history_scroll_down()
        }
    }

    // start input reception
    pub fn start_reception(&mut self) {
        while !self.done {
            match self.rx.try_recv() {
                Ok(Event::OutputUpdate(_cmd)) => self.output_update(_cmd),
                Ok(Event::Exit) => self.done = true,
                Ok(Event::Input(i)) => {
                    match i {
                        // Screen Resize
                        KEY_RESIZE => self.watch.resize(),

                        // watch pad up/down
                        KEY_UP => self.watch.window_scroll_up(), // Up
                        KEY_DOWN => self.watch.window_scroll_down(), // Down

                        // history pad up/down
                        KEY_SR => self.history_scroll_up(), // Shift + Up
                        KEY_SF => self.history_scroll_down(), // Shift + Down

                        // exit this program
                        KEY_F1 | 0x1b | 0x71 => self.exit(), // ESC(0x1b),q(0x71),F1

                        _ => {}
                    }
                }
             _ => {}
            };
        }
    }
}
