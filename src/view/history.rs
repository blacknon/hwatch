extern crate ncurses;

use std::sync::Mutex;
use ncurses::*;

use cmd::Result;

pub struct History {
    pub count: i32,
    pub latest_result_status: bool,

    pub history: Mutex<Vec<Result>>,
    pub history_pad: ncurses::WINDOW,
    pub history_pad_lines: i32,
    pub history_pad_position: i32,
    pub history_position: i32,

    pub selected_position: i32, // select position

    pub screen: ncurses::WINDOW,
}

impl History {
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        Self {
            count: 0,
            latest_result_status: true,

            history : Mutex::new(vec![]),
            history_pad: newpad(0,0),
            history_pad_lines: 0,
            history_pad_position: 0,
            history_position: 0,

            selected_position: 0, 

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

        // define history pad line
        if self.count.clone() > max_y - 2 {
            self.history_pad_lines = self.count.clone();
        } else {
            self.history_pad_lines = max_y - 2;
        }
        
        // Create history_pad
        self.history_pad = newpad(self.history_pad_lines, max_x);

        // print latest
        let _latest_status = self.latest_result_status;
        self.print_history(0, "latest             ".to_string(), _latest_status);

        // print history
        let mut _history = self.history.lock().unwrap().clone();

        let mut i = 1;
        let _length = _history.len();
        for x in (0.._length).rev() {
            let _timestamp = _history[x].timestamp.clone();
            let _status = _history[x].status.clone();

            self.print_history(i, _timestamp, _status);
            i += 1;
        }

        prefresh(self.history_pad, self.history_pad_position, 0, 2, max_x - 21, max_y - 1, max_x - 1);
    }

    fn print_history(&mut self, position: i32, word: String, status: bool) {
        if position == self.selected_position {
            if status == true {
                // selected line and status true
                wattron(self.history_pad,A_REVERSE()|COLOR_PAIR(2));
                wprintw(self.history_pad, &format!(">{}\n", word));
                wattroff(self.history_pad,A_REVERSE()|COLOR_PAIR(2));
            } else {
                // selected line and status false
                wattron(self.history_pad,A_REVERSE()|COLOR_PAIR(3));
                wprintw(self.history_pad, &format!(">{}\n", word));
                wattroff(self.history_pad,A_REVERSE()|COLOR_PAIR(3));
            }
        } else {
            if status == true {
                // not selected line and status true
                wattron(self.history_pad,COLOR_PAIR(2));
                wprintw(self.history_pad, &format!(" {}\n", word));
                wattroff(self.history_pad,COLOR_PAIR(2));
            } else {
                // not selected line and status false
                wattron(self.history_pad,COLOR_PAIR(3));
                wprintw(self.history_pad, &format!(" {}\n", word));
                wattroff(self.history_pad,COLOR_PAIR(3));
            }
        }
    }

    pub fn scroll_up(&mut self){
        if self.selected_position > 0 {
            self.selected_position -= 1;
            self.draw_history_pad();
            
        }
    }

    pub fn scroll_down(&mut self){
        if self.count > self.selected_position {
            self.selected_position += 1;
            self.draw_history_pad();

        }
    }



    pub fn append_history(&mut self, _result: Result) {
        let mut history = self.history.lock().unwrap();
        history.push(_result);
        self.count += 1;
    }

    pub fn exit(&mut self) {
        delwin(self.history_pad);
    }
}