extern crate ncurses;

use std::sync::Mutex;
use ncurses::*;

use cmd::Result;

pub struct History {
    pub count: i32,

    pub history: Mutex<Vec<Result>>,
    pub history_pad: ncurses::WINDOW,
    pub history_pad_lines: i32,
    pub history_pad_position: i32,
    pub history_position: i32,

    pub pad_position: i32,

    pub screen: ncurses::WINDOW,
}

impl History {
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        Self {
            count: 0,

            history : Mutex::new(vec![]),
            history_pad: newpad(0,0),
            history_pad_lines: 0,
            history_pad_position: 0,
            history_position: 0,

            pad_position: 0,

            screen: _screen,
        }
    }

    pub fn get_latest_history(&mut self) -> Result {
        let mut _result = Result::new();

        let mut _history = self.history.lock().unwrap();
        let _length = _history.len();
        if _length >= 1 {
            _result = _history[_length - 1].clone();
        }
        return _result;
    }

    pub fn draw_history_pad(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        refresh();
        if self.count.clone() > max_y - 3 {
            self.history_pad_lines = self.count.clone();
        } else {
            self.history_pad_lines = max_y - 3;
        }

        self.history_pad = newpad(self.history_pad_lines, max_x);
        wbkgd(self.history_pad,COLOR_PAIR(11));

        let mut _history = self.history.lock().unwrap();
        let _length = _history.len();
        for x in 0.._length {
            wprintw(self.history_pad, &format!("{}\n", _history[x].timestamp.clone()));
        }

        prefresh(self.history_pad, self.history_pad_position, 0, 2, max_x - 20, max_y - 1, max_x - 1);
    }


    pub fn append_history(&mut self, _result: Result) {
        let mut history = self.history.lock().unwrap();
        history.push(_result);
        self.count += 1;
    }





    // pub fn start_pad(&mut self) {
    //     let mut max_x = 0;
    //     let mut max_y = 0;
    //     getmaxyx(self.screen, &mut max_y, &mut max_x);

    //     refresh();
    //     // let pad_lines = self.count;
    //     let pad_lines = 8;
    //     self.history_pad = newpad(pad_lines, max_x);

    //     wbkgd(self.history_pad,COLOR_PAIR(11));

    //     wprintw(self.history_pad, &format!("{}\n", "2018/07/05 18:22:22"));
    //     wprintw(self.history_pad, &format!("{}", "\n".to_string()));
    //     wprintw(self.history_pad, &format!("{}", "2018/07/05 18:22:23"));
    //     wprintw(self.history_pad, &format!("{}", "\n".to_string()));
    //     wprintw(self.history_pad, &format!("{}", "2018/07/05 18:22:24"));
    //     wprintw(self.history_pad, &format!("{}", "\n".to_string()));
    //     wprintw(self.history_pad, &format!("{}", "2018/07/05 18:22:25"));
    //     wprintw(self.history_pad, &format!("{}", "\n".to_string()));
    //     prefresh(self.history_pad, self.history_pad_position, 0, 2, max_x - 20, max_y - 1, max_x - 1);
    // }

    pub fn exit(&mut self) {
        delwin(self.history_pad);
    }
}