// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): 以下の情報を参考に開発を進めていく！
//   - 【参考】
//     - http://www.kis-lab.com/serikashiki/man/ncurses.html#output

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
    show_help_win: bool,
    pub help_win: WINDOW,
    pub selected: i32, // history select position
    pub screen: WINDOW,
}

impl Watch {
    // set default value
    pub fn new(_screen: WINDOW) -> Self {
        // Create WatchPad
        let _watch = WatchPad::new(_screen.clone());
        Self {
            diff: ::DIFF_DISABLE,
            color: false,
            output_type: ::IS_OUTPUT,
            count: 0,
            latest_result: Result::new(),
            watch_pad: _watch,
            history: Mutex::new(vec![]),
            history_pad: newpad(0, 0),
            history_pad_position: 0,
            show_help_win: false,
            help_win: newwin(0, 0, 0, 0),
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

        // set history info
        let mut i = 1;
        let mut _history = self.history.lock().unwrap().clone();
        let _length = _history.len();

        // print history
        for x in 0.._length {
            let _timestamp = _history[x].timestamp.clone();
            let _status = _history[x].status.clone();

            self.print_history(i, _timestamp, _status);
            i += 1;
        }

        // adjust self.history_pad_position at history pad last line.
        let _history_pad_lastline = self.history_pad_position + max_y - 4;
        if self.selected >= _history_pad_lastline {
            self.history_pad_position = self.selected - max_y + 3;

            if self.history_pad_position < 0 {
                self.history_pad_position = 0;
            }
        }

        // adjust self.history_pad_position at history pad first line.
        let _history_pad_firstline = self.history_pad_position;
        if self.selected <= _history_pad_firstline {
            self.history_pad_position = self.selected;

            if self.history_pad_position < 0 {
                self.history_pad_position = 0;
            }
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

    pub fn draw_help(&mut self) {
        // Set help text
        let mut _help_text = format!("{}", "[h] key   ... show this help message.");
        let mut _help_text = format!("{}\n {}", _help_text, "[c] key   ... toggle color mode.");
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[d] key   ... switch diff mode at None, Watch, Line mode."
        );
        let mut _help_text = format!("{}\n {}", _help_text, "[q] key   ... exit hwatch.");
        let mut _help_text = format!("{}\n {}", _help_text, "[0] key   ... disable diff.");
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[1] key   ... switch Watch type diff."
        );
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[2] key   ... switch Line type diff."
        );
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[F1] key  ... change output mode as stdout."
        );
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[F2] key  ... change output mode as stderr."
        );
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[F3] key  ... change output mode as stdout/stderr set."
        );
        let mut _help_text = format!(
            "{}\n {}",
            _help_text, "[Tab] key ... toggle current pad at history, watchpad."
        );

        // TODO(blacknon): help messageのサイズに応じてhelp_winのサイズを設定
        // TODO(blacknon): help_winをターミナル中央に表示するように指定

        // Create help_window
        self.help_win = newwin(40, 100, 5, 5);

        // Write help text
        wmove(self.help_win, 1, 1);
        waddstr(self.help_win, &format!("{}", _help_text));

        // Write box at self.help_win
        box_(self.help_win, 0, 0);

        // refresh and overlay
        wrefresh(self.help_win);
        overlay(self.help_win, self.screen);
    }

    pub fn toggle_help_window(&mut self) {
        // create help pad window
        if !self.show_help_win {
            self.show_help_win = true;

            self.draw_help();
        } else {
            self.show_help_win = false;

            // delete exist help window
            delwin(self.help_win);

            // create empty window
            self.help_win = newwin(0, 0, 0, 0);

            // refresh and overlay
            wrefresh(self.help_win);
            overlay(self.screen, self.help_win);
        }
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
            waddstr(self.history_pad, &_print_data);
            wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(COLORSET_G_D));
        } else {
            // selected line and status false
            wattron(self.history_pad, COLOR_PAIR(COLORSET_R_D));
            waddstr(self.history_pad, &_print_data);
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

    // TODO(blacknon): 表示範囲を外れた場合は`self.watch.history_pad_position`の位置を調整する
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

        // TODO(blacknon): result_dataの値をBase64からTextに戻す処理を追加する(log対応)

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
                let mut diff = diff::LineDiff::new(self.color);
                diff.create_dataset(before_data, target_data);
                self.watchpad_create(diff.line + 1);

                diff.print(self.watch_pad.clone());
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
