mod diff;
mod window;

use std::sync::Mutex;
use ncurses::*;

use cmd::Result;
use self::window::WatchPad;
use view::*;

pub struct Watch {
    pub diff: i32,
    pub output_type: i32,

    pub count: i32,
    pub latest_result: Result,
    pub before_result: Result,

    pub watchpad: self::window::WatchPad,

    pub history: Mutex<Vec<Result>>,
    pub history_pad: WINDOW,
    pub history_pad_lines: i32,
    pub history_pad_position: i32,

    pub selected_position: i32, // history select position

    pub screen: WINDOW,
}

impl Watch {
    // set default value
    pub fn new(_screen: WINDOW, _diff: i32) -> Self {
        let _watch = WatchPad::new(_screen.clone());
        Self {
            diff: _diff,
            output_type: IS_OUTPUT,

            count: 0,
            latest_result: Result::new(),
            before_result: Result::new(),

            watchpad: _watch,

            history : Mutex::new(vec![]),
            history_pad: newpad(0,0),
            history_pad_lines: 0,
            history_pad_position: 0,

            selected_position: 0, 

            screen: _screen,
        }
    }

    pub fn get_latest_history(&mut self) -> Result {
        let mut _result = Result::new();

        let mut _history = self.history.lock().unwrap();
        let _length = _history.len();
        if _length >= 1 {
            _result = _history[0].clone();
        }
        return _result;
    }

    pub fn draw_history_pad(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        refresh();
        
        // Create history_pad
        self.history_pad_lines = self.count.clone() + 1;
        self.history_pad = newpad(self.history_pad_lines, max_x);

        // print latest
        let _latest_status = self.latest_result.status;
        self.print_history(0, "latest             ".to_string(), _latest_status);

        // print history
        let mut _history = self.history.lock().unwrap().clone();

        let mut i = 1;
        let _length = _history.len();
        for x in 0.._length {
            let _timestamp = _history[x].timestamp.clone();
            let _status = _history[x].status.clone();

            self.print_history(i, _timestamp, _status);
            i += 1;
        }
        let history_pad_lastline = self.history_pad_position + max_y - 4;
        if self.selected_position >= history_pad_lastline {
            self.history_pad_position = self.selected_position - max_y + 3;
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

    pub fn history_scroll_up(&mut self) {
        if self.selected_position > 0 {
            self.selected_position -= 1;
            self.draw_history_pad();
            self.watch_update();
        }
    }

    pub fn history_scroll_down(&mut self) {
        if self.count > self.selected_position {
            self.selected_position += 1;
            self.draw_history_pad();
            self.watch_update();
        }
    }

    pub fn window_scroll_up(&mut self) {
        self.watchpad.scroll_up()
    }

    pub fn window_scroll_down(&mut self) {
        self.watchpad.scroll_down();
    }

    pub fn resize(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        resizeterm(max_y,max_x);
        
        self.draw_history_pad();
        self.watchpad.resize();
    }

    pub fn append_history(&mut self, _result: Result) {
        let mut history = self.history.lock().unwrap();
        history.insert(0, _result);
        self.count += 1;
    }

    pub fn watch_update(&mut self) {
        let count = self.count.clone();

        if self.diff != 0 && count > 1 {
            self.diff_watch_update();
        } else {
            self.plane_watch_update();
        }

        self.watchpad.draw_output_pad();
        self.draw_history_pad();
    }

    fn plane_watch_update(&mut self) {
        let target_result = self.get_target_result(0);

        self.watchpad.result = target_result.clone();
        self.watchpad.before_update_output_pad();
        self.watchpad.update_output_pad_text(self.diff);
    }

    fn diff_watch_update(&mut self) {
        let before_result = self.get_target_result(-1);
        let target_result = self.get_target_result(0);

        if target_result.output != before_result.output && self.selected_position != self.count {
            self.watchpad.result = target_result.clone();
            match self.diff {
                1 => self.watch_diff_print(before_result, target_result),
                2 => self.line_diff_print(before_result,target_result),
                _ => self.plane_watch_update(),
            }
        } else {
            self.watchpad.result = target_result.clone();
            self.watchpad.before_update_output_pad();
            self.watchpad.update_output_pad_text(self.diff);
        }
    }

    fn watch_diff_print(&mut self,before_result: Result,target_result: Result) {
        self.watchpad.before_update_output_pad();
        diff::watch_diff(
            self.watchpad.clone(),
            before_result.output,
            target_result.output
        );
    }

    fn line_diff_print(&mut self,before_result: Result,target_result: Result) {
        let line_diff_str = diff::line_diff_str_get(before_result.output.clone(),target_result.output.clone());
        self.watchpad.result_diff_output = line_diff_str;
        self.watchpad.before_update_output_pad();
        diff::line_diff(
            self.watchpad.clone(),
            before_result.output,
            target_result.output
        );
        self.watchpad.result_diff_output = String::new();
    }

    // @note:
    //    [get_type value]
    //    0 ... target result
    //   -1 ... before result
    //    1 ... next result
    fn get_target_result(&mut self, get_type: i32) ->Result {
        let mut result = Result::new();
        if self.selected_position != 0 {
            let mut _history = self.history.lock().unwrap().clone();
            let _length = _history.len();

            let mut i = 1;
            for x in 0.._length {
                if get_type == 0 && i == self.selected_position {
                    result = _history[x].clone();
                } else if get_type == -1 && i == self.selected_position + 1 {
                    result = _history[x].clone();
                } else if get_type == 1 && i == self.selected_position - 1 {
                    result = _history[x].clone();
                }
                i += 1;
            }
        } else {
            if get_type == 0 {
                result = self.latest_result.clone();
            } else if get_type == -1 {
                result = self.before_result.clone();
            }
        }
        return result
    }


    fn get_result_output(&mut self,_result: Result) ->String {
        let _result_output =  String::new();

        return _result_output
    }

    pub fn exit(&mut self) {
        self.watchpad.exit();
        delwin(self.history_pad);
    }
}