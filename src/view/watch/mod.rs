// @TODO
//     WatchPad側には表示機能だけを渡し、出力内容の制御は本ファイルで行うよう書き換えが必要！
//     watch_pad側にあるcreate_padも、こちら側に移してきたほうがいいかもしれない…(´・ω・｀)
//
//
//

// module
use ncurses::*;
use std::sync::Mutex;

// local module
mod ansi;
mod diff;
mod watch;
use self::watch::WatchPad;
use cmd::Result;

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
            wattron(self.history_pad, COLOR_PAIR(2));
            wprintw(self.history_pad, &_print_data);
            wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(2));
        } else {
            // selected line and status false
            wattron(self.history_pad, COLOR_PAIR(3));
            wprintw(self.history_pad, &_print_data);
            wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(3));
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
        // count history
        let history_count = self.count;
        if history_count > 1 {
            match self.diff {
                ::DIFF_DISABLE => self.watchpad_plane_update(),
                ::DIFF_WATCH | ::DIFF_LINE => self.watchpad_diff_update(),
                _ => {}
            }
        } else {
            self.watchpad_plane_update()
        }

        // watch_pad update
        self.watch_pad.draw_output_pad(); // watch_pad.update()に統合でよさそう

        // history update
        self.draw_history();
    }

    // @TODO: add color
    fn watchpad_plane_update(&mut self) {
        let target_result = self.get_result(0);
        let target_result_data = self.get_output(target_result.clone());

        self.watch_pad.result = target_result.clone();
        self.watch_pad.create_pad(self.output_type);
        // self.watch_pad.update(self.diff, self.output_type);
        self.watch_pad.print_plain(target_result_data);
    }

    // @TODO: add color
    fn watchpad_diff_update(&mut self) {
        let target_result = self.get_result(0);
        let before_result = self.get_result(1);
        let target_result_data = self.get_output(target_result.clone());
        let before_result_data = self.get_output(before_result.clone());

        self.watch_pad.result = target_result.clone();

        // @MEMO:
        //     get_outputからｃolor付きで文字列を取得して、それをforで回せばいいのか？？
        match self.diff {
            ::DIFF_WATCH => {
                self.watch_pad.create_pad(self.output_type);
                diff::watch_diff(
                    self.watch_pad.clone(),
                    before_result_data,
                    target_result_data,
                )
            }
            ::DIFF_LINE => {
                let line_diff_str =
                    diff::line_diff_str_get(before_result_data.clone(), target_result_data.clone());
                self.watch_pad.result_diff_output = line_diff_str;
                self.watch_pad.create_pad(self.output_type);
                diff::line_diff(
                    self.watch_pad.clone(),
                    before_result_data,
                    target_result_data,
                )
            }
            _ => {}
        }
    }

    // fn diff_watch_update(&mut self) {
    //     let mut _before_result_data = self.get_output(before_result);
    //     let mut _target_result_data = self.get_output(target_result);

    //     if target_result.output != before_result.output && self.selected != self.count {
    //         self.watch_pad.result = target_result.clone();
    //         match self.diff {
    //             1 => self.watch_diff_print(before_result, target_result),
    //             2 => self.line_diff_print(before_result, target_result),
    //             _ => self.plane_watch_update(),
    //         }
    //     } else {
    //         self.watch_pad.result = target_result.clone();
    //         self.watch_pad.create_pad(self.output_type);
    //         self.watch_pad.update(self.diff, self.output_type);
    //     }
    // }

    // fn watch_diff_print(&mut self, before_result: Result, target_result: Result) {
    //     let mut _before_result_data = self.get_output(before_result);
    //     let mut _target_result_data = self.get_output(target_result);

    //     self.watch_pad.create_pad(self.output_type);
    //     diff::watch_diff(
    //         self.watch_pad.clone(),
    //         _before_result_data,
    //         _target_result_data,
    //     );
    // }

    // fn line_diff_print(&mut self, before_result: Result, target_result: Result) {
    //     let _before_result_data = self.get_output(before_result);
    //     let _target_result_data = self.get_output(target_result);

    //     let line_diff_str =
    //         diff::line_diff_str_get(_before_result_data.clone(), _target_result_data.clone());
    //     self.watch_pad.result_diff_output = line_diff_str;
    //     self.watch_pad.create_pad(self.output_type);
    //     diff::line_diff(
    //         self.watch_pad.clone(),
    //         _before_result_data,
    //         _target_result_data,
    //     );
    //     self.watch_pad.result_diff_output = String::new();
    // }

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

        // @TEST!!!
        // let text = b"test ";
        // println!("{:?}", ansi::get_ansi_iter(text));
    }
}
