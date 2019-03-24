// module
use ncurses::*;
use std::env;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

// local module
mod header;
mod watch;
use self::watch::Watch;
use cmd::Result;
use event::Event;

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
        init_pair(4, COLOR_YELLOW, -1); // fg=yellow, bg=clear
        init_pair(5, COLOR_CYAN, -1); // fg=cyan, bg=clear
        init_pair(6, COLOR_MAGENTA, -1); // fg=cyan, bg=clear
        init_pair(11, COLOR_BLACK, COLOR_WHITE); // fg=black, bg=white
        init_pair(12, COLOR_WHITE, COLOR_RED); // fg=white, bg=red
        init_pair(13, COLOR_WHITE, COLOR_GREEN); // fg=white, bg=green
        init_pair(14, COLOR_WHITE, COLOR_YELLOW); // fg=white, bg=green
        init_pair(15, COLOR_BLACK, COLOR_CYAN); // fg=white, bg=green

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
            rx: rx,
        }
    }

    fn exit(&mut self) {
        self.watch.exit();
        let _ = self.tx.send(Event::Exit);
    }

    fn update(&mut self, _result: Result) {
        // set header diff flag
        self.header.diff = self.watch.diff;

        // update latest result
        self.watch.latest_result = _result.clone();

        // history append result
        if self.watch.get_latest_history().output != _result.output {
            clear();
            self.header.result = _result.clone();
            self.header.update();
            self.watch.append_history(_result.clone());

            // add selected positon
            if self.watch.selected != 0 {
                self.watch.selected += 1;
            }
            self.watch.draw_history();
            self.watch.update();
        }

        // if history selected latest, update watch window.
        if self.watch.selected == 0 {
            self.header.result = _result.clone();
            self.header.update();
            self.watch.draw_history();
            self.watch.update();
        } else {
            self.header.result = _result.clone();
            self.header.update();
        }
    }

    fn toggle_diff(&mut self) {
        // add num
        let mut now_diff = self.watch.diff;
        now_diff += 1;

        self.switch_diff(now_diff % 3);
    }

    fn switch_diff(&mut self, _diff: i32) {
        // set value
        self.watch.diff = _diff;
        self.header.diff = self.watch.diff;

        // draw
        clear();
        self.header.update();
        self.watch.draw_history();
        self.watch.update();
    }

    fn toggle_pad(&mut self) {
        // add num
        let mut now_pad = self.header.active_pad;
        now_pad += 3;

        self.header.active_pad = now_pad % 2;
        self.header.update();
    }

    fn up(&mut self) {
        match self.header.active_pad {
            ::IS_WATCH_PAD => self.watch.window_up(),
            ::IS_HISTORY_PAD => {
                if self.watch.selected > 0 {
                    clear();
                    self.header.update();
                    self.watch.history_up();
                }
            }
            _ => (),
        }
    }

    fn down(&mut self) {
        match self.header.active_pad {
            ::IS_WATCH_PAD => self.watch.window_down(),
            ::IS_HISTORY_PAD => {
                if self.watch.count > self.watch.selected {
                    clear();
                    self.header.update();
                    self.watch.history_down()
                }
            }
            _ => (),
        }
    }

    fn set_output_type(&mut self, _output_type: i32) {
        // set value
        self.watch.output_type = _output_type;
        self.header.output = _output_type;

        // draw
        clear();
        self.header.update();
        self.watch.draw_history();
        self.watch.update();
    }

    // start input reception
    pub fn start_reception(&mut self) {
        mousemask(ALL_MOUSE_EVENTS as mmask_t, None);
        while !self.done {
            thread::sleep(Duration::from_millis(10));
            match self.rx.try_recv() {
                Ok(Event::OutputUpdate(_cmd)) => self.update(_cmd),
                Ok(Event::Exit) => self.done = true,
                Ok(Event::Signal(i)) => match i {
                    0 => {}
                    0x02 => self.exit(),
                    _ => {}
                },
                Ok(Event::Input(i)) => self.input_action(i),
                _ => {}
            };
        }
    }

    fn input_action(&mut self, _input: i32) {
        match _input {
            // Mouse
            KEY_MOUSE => {
                let mut mevent = MEVENT {
                    id: 0,
                    x: 0,
                    y: 0,
                    z: 0,
                    bstate: 0,
                };
                let _error = getmouse(&mut mevent);
                if _error == 0 {
                    self.mouse_action(mevent)
                }
            }

            // Screen Resize
            KEY_RESIZE => self.watch.resize(),

            // change active pad
            0x09 => self.toggle_pad(), // Tab

            // pad up/down
            KEY_UP => self.up(),     // Arrow Up
            KEY_DOWN => self.down(), // Arrow Down

            // change diff mode
            0x64 => self.toggle_diff(),  // d(0x64)
            0x30 => self.switch_diff(0), // 0(0x30)
            0x31 => self.switch_diff(1), // 1(0x31)
            0x32 => self.switch_diff(2), // 2(0x32)

            // change output
            KEY_F1 => self.set_output_type(::IS_STDOUT),
            KEY_F2 => self.set_output_type(::IS_STDERR),
            KEY_F3 => self.set_output_type(::IS_OUTPUT),

            // exit this program
            0x1b | 0x71 => self.exit(), // ESC(0x1b),q(0x71)

            _ => {}
        }
    }

    fn mouse_action(&mut self, _mevent: MEVENT) {
        let _mouse_event = _mevent.bstate as i32;
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        // mouse is not on header
        if _mevent.y > 1 {
            match _mouse_event {
                // mouse left button click
                BUTTON1_CLICKED => {
                    if max_x - ::HISTORY_WIDTH < _mevent.x {
                        let _mouse_select_line = _mevent.y - 2 + self.watch.history_pad_position;

                        if self.watch.count >= _mouse_select_line {
                            self.watch.selected = _mouse_select_line;

                            // update draw
                            self.watch.draw_history();
                            self.watch.update();
                        }
                    }
                }

                // mouse wheel up
                BUTTON4_PRESSED => {
                    if max_x - ::HISTORY_WIDTH < _mevent.x {
                        self.header.active_pad = ::IS_HISTORY_PAD;
                    } else {
                        self.header.active_pad = ::IS_WATCH_PAD;
                    }
                    self.header.update();
                    self.up();
                }

                // mouse wheel down
                BUTTON5_PRESSED => {
                    if max_x - ::HISTORY_WIDTH < _mevent.x {
                        self.header.active_pad = ::IS_HISTORY_PAD;
                    } else {
                        self.header.active_pad = ::IS_WATCH_PAD;
                    }
                    self.header.update();
                    self.down();
                }
                _ => {}
            }
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
