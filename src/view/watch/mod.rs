// module
use ncurses::*;
use std::sync::Mutex;

// local module
mod diff;
mod watch;
use self::watch::WatchPad;
use cmd::Result;
use view::color::*;

pub struct Watch {
    pub diff: i32,
    pub color: bool,
    pub output_type: i32,
    pub count: i32,
    pub latest_result: Result,
    pub watch_pad: WatchPad,
    pub history: Mutex<Vec<Result>>,
    pub history_pad: WINDOW,
    pub history_pad_position: i32,
    pub selected: i32, // history select position
    pub screen: WINDOW,
}

impl Watch {
    // set default value
    pub fn new(_screen: WINDOW, _diff: i32, _color: bool) -> Self {
        // Create WatchPad
        let _watch = WatchPad::new(_screen.clone());
        Self {
            diff: _diff,
            color: _color,
            output_type: ::IS_OUTPUT,
            count: 0,
            latest_result: Result::new(),
            watch_pad: _watch,
            history: Mutex::new(vec![]),
            history_pad: newpad(0, 0),
            history_pad_position: 0,
            selected: 0,
            screen: _screen,
        }
    }

    pub fn draw_history(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        refresh();

        // Create history_pad
        self.history_pad = newpad(self.count + 1, max_x);

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
        let _history_pad_lastline = self.history_pad_position + max_y - 4;
        if self.selected >= _history_pad_lastline {
            self.history_pad_position = self.selected - max_y + 3;
        }

        prefresh(
            self.history_pad,
            self.history_pad_position,
            0,
            2,
            max_x - ::HISTORY_WIDTH,
            max_y - 1,
            max_x - 1,
        );
    }

    fn print_history(&mut self, position: i32, word: String, status: bool) {
        let mut _print_data = String::new();
        if position == self.selected {
            wattron(self.history_pad, A_REVERSE());
            _print_data = format!(">{}\n", word);
        } else {
            _print_data = format!(" {}\n", word);
        }

        if status == true {
            // selected line and status true
            wattron(self.history_pad, COLOR_PAIR(COLORSET_G_D));
            wprintw(self.history_pad, &_print_data);
            wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(COLORSET_G_D));
        } else {
            // selected line and status false
            wattron(self.history_pad, COLOR_PAIR(COLORSET_R_D));
            wprintw(self.history_pad, &_print_data);
            wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(COLORSET_R_D));
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

    pub fn history_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.draw_history();
            self.update();
        }
    }

    pub fn history_down(&mut self) {
        if self.count > self.selected {
            self.selected += 1;
            self.draw_history();
            self.update();
        }
    }

    pub fn window_up(&mut self) {
        self.watch_pad.scroll_up()
    }

    pub fn window_down(&mut self) {
        self.watch_pad.scroll_down();
    }

    pub fn resize(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        resizeterm(max_y, max_x);

        self.draw_history();
        self.watch_pad.resize();
    }

    pub fn append_history(&mut self, _result: Result) {
        let mut history = self.history.lock().unwrap();
        history.insert(0, _result);
        self.count += 1;
    }

    // update watch,history pads
    pub fn update(&mut self) {
        // watchpad update
        match self.diff {
            ::DIFF_DISABLE => self.watchpad_plain_update(),
            ::DIFF_WATCH | ::DIFF_LINE => self.watchpad_diff_update(),
            _ => {}
        }
        self.watch_pad.draw_output_pad();

        // history update
        self.draw_history();
    }

    // plane update
    fn watchpad_plain_update(&mut self) {
        let result = self.get_result(0);
        let result_data = self.get_output(result.clone());

        let watchpad_size = self.watchpad_get_size(result_data.clone());
        self.watchpad_create(watchpad_size);

        if self.color {
            let ansi_data = ansi_parse(&result_data);
            for data in ansi_data {
                let front_color = data.ansi.1 as i16;
                let back_color = data.ansi.2 as i16;

                self.watch_pad
                    .print(data.data, front_color, back_color, vec![data.ansi.0])
            }
        } else {
            // color disable
            self.watch_pad
                .print(result_data, COLOR_ELEMENT_D, COLOR_ELEMENT_D, vec![]);
        }
    }

    // @TODO: add color (v1.0.0)
    // @NOTE:
    fn watchpad_diff_update(&mut self) {
        let target_result = self.get_result(0);
        let before_result = self.get_result(1);
        let target_data = self.get_output(target_result.clone());
        let before_data = self.get_output(before_result.clone());

        match self.diff {
            ::DIFF_WATCH => {
                // set watchpad size
                let watchpad_size = self.watchpad_get_size(target_data.clone());
                self.watchpad_create(watchpad_size);

                diff::watch_diff(self.watch_pad.clone(), before_data, target_data, self.color);
            }
            ::DIFF_LINE => {
                // set watchpad size
                // @TODO: Refactoring (line_diff_str)
                let line_diff_str =
                    diff::line_diff_str_get(before_data.clone(), target_data.clone());
                let watchpad_size = self.watchpad_get_size(line_diff_str.clone());
                self.watchpad_create(watchpad_size + 1);

                diff::line_diff(self.watch_pad.clone(), before_data, target_data, self.color)
            }
            _ => {}
        }
    }

    // get watchpad size
    fn watchpad_get_size(&mut self, data: String) -> i32 {
        // get screen size
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        // set watchpad width
        let watchpad_width = max_x - (::HISTORY_WIDTH + 2);

        let mut count: i32 = 0;
        let lines = data.split("\n");

        for l in lines {
            // @TODO: Add Color line count
            if self.color {
                let color_pair = ansi_parse(&l.to_string());
                let mut linestr = "".to_string();
                for pair in color_pair {
                    linestr = [linestr, pair.data].concat();
                }
                count += count_line(linestr.to_string(), watchpad_width.clone());
            } else {
                count += count_line(format!("{:?}", l).to_string(), watchpad_width.clone());
            }
        }

        if self.color {
            count += 1;
        }

        return count;
    }

    fn watchpad_create(&mut self, size: i32) {
        // get screen size
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        // set watchpad width
        let watchpad_width = max_x - (::HISTORY_WIDTH + 2);

        // create watchpad
        self.watch_pad.pad_lines = size;
        self.watch_pad.pad = newpad(size, watchpad_width);
    }

    // get output string
    fn get_output(&mut self, result: Result) -> String {
        let mut output = String::new();
        match self.output_type {
            ::IS_OUTPUT => output = result.output.clone(),
            ::IS_STDOUT => output = result.stdout.clone(),
            ::IS_STDERR => output = result.stderr.clone(),
            _ => {}
        };
        return output;
    }

    // @note:
    //    [get_type value]
    //    0 ... target result
    //    1 ... before result
    //   -1 ... next result
    fn get_result(&mut self, get_type: i32) -> Result {
        let mut result = Result::new();
        let mut _history = self.history.lock().unwrap().clone();

        let mut results_vec = vec![self.latest_result.clone()];
        results_vec.append(&mut _history);

        let increment = (self.selected + get_type) as usize;
        if results_vec.len() > increment {
            result = results_vec[increment].clone();
        }

        return result;
    }

    pub fn exit(&mut self) {
        self.watch_pad.exit();
        delwin(self.history_pad);
    }
}

// get lines in watchpad
fn count_line(_string: String, _width: i32) -> i32 {
    let char_vec: Vec<char> = _string.chars().collect();
    let mut _char_count = 0;
    let mut _line_count = 1;

    for ch in char_vec {
        if ch.to_string().len() > 1 {
            _char_count += 2;
        } else {
            _char_count += 1;
        }

        if _char_count > _width {
            _line_count += 1;
            _char_count = 0;
        }
    }
    return _line_count;
}
