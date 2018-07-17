mod header;
mod watch;

use std::env;
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
        // Set locale
        let locale_conf = LcCategory::all;
        let lang = get_lang();
        setlocale(locale_conf, &lang);

        // Create ncurses screen
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
        init_pair(4, COLOR_YELLOW, -1); // fg=red, bg=clear
        init_pair(11, COLOR_BLACK, COLOR_WHITE); // fg=black, bg=white
        init_pair(12, COLOR_WHITE, COLOR_RED); // fg=white, bg=red
        init_pair(13, COLOR_WHITE, COLOR_GREEN); // fg=white, bg=green
        init_pair(14, COLOR_WHITE, COLOR_YELLOW); // fg=white, bg=green

        let mut diff_type = 0;
        if _diff {
            diff_type = 1;
        }

        let _watch = Watch::new(_screen.clone(), diff_type);
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
        println!("{}", self.watch.latest_result.output);
    }

    fn output_update(&mut self, _result: Result) {
        // set header diff flag
        self.header.diff = self.watch.diff;

        // update before result
        let before_result = self.watch.latest_result.clone();
        self.watch.before_result = before_result;

        // update latest result
        self.watch.latest_result = _result.clone();

        // history append result
        if self.watch.get_latest_history().output != _result.output {
            clear();
            self.header.result = _result.clone();
            self.header.update_header();
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
            self.header.result = _result.clone();
            self.header.update_header();
            self.watch.draw_history_pad();
            self.watch.watch_update();
        } else {
            self.header.result = _result.clone();
            self.header.update_header();
        }
    }

    fn history_scroll_up(&mut self) {
        if self.watch.selected_position > 0 {
            clear();
            self.header.update_header();
            self.watch.history_scroll_up()
        }
    }

    fn history_scroll_down(&mut self) {
        if self.watch.count > self.watch.selected_position {
            clear();
            self.header.update_header();
            self.watch.history_scroll_down()
        }
    }

    fn toggle_diff(&mut self) {
        // add number
        let mut now_diff = self.watch.diff;
        now_diff += 1;

        // switch and show diff mode
        match now_diff % 3 {
            0 => self.switch_disable_diff(),
            1 => self.switch_watch_diff(),
            2 => self.switch_line_diff(),
            _ => (),
        }
    }

    fn switch_disable_diff(&mut self) {
        self.watch.diff = 0;
        self.header.diff = self.watch.diff;
        clear();
        self.header.update_header();
        self.watch.draw_history_pad();
        self.watch.watch_update();
    }

    fn switch_watch_diff(&mut self) {
        self.watch.diff = 1;
        self.header.diff = self.watch.diff;
        clear();
        self.header.update_header();
        self.watch.draw_history_pad();
        self.watch.watch_update();       
    }

    fn switch_line_diff(&mut self) {
        self.watch.diff = 2;
        self.header.diff = self.watch.diff;
        clear();
        self.header.update_header();
        self.watch.draw_history_pad();
        self.watch.watch_update();
    }

    // fn switch_word_diff(&mut self) {
        
    // }

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

                        // change diff mode
                        0x64 => self.toggle_diff(), // d(0x64)
                        0x30 => self.switch_disable_diff(), // 0(0x30)
                        0x31 => self.switch_watch_diff(), // 1(0x31)
                        0x32 => self.switch_line_diff(), // 2(0x32)
                        // 0x33 => // 3(0x33)

                        // change output
                        // KEY_F1 => // Stdout only (F1 key)
                        // KEY_F2 => // Stderr only (F2 key)
                        // KEY_F3 => // Stdout and Stderr (F1 key)

                        // exit this program
                        0x1b | 0x71 => self.exit(), // ESC(0x1b),q(0x71),F1

                        _ => {}
                    }
                }
             _ => {}
            };
        }
    }
}


fn get_lang() -> String {
    let key = "LANG";
    match env::var(key) {
        Ok(val) => return val,
        _ => return String::new(),
    }
}