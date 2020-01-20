// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use ncurses::*;

// local module
use cmd::Result;
use view::color::*;

pub struct Header {
    pub screen: ncurses::WINDOW,
    pub result: Result,
    pub color: bool,
    pub diff: i32,
    pub interval: f64,
    pub command: String,
    pub output: i32,
    pub active_pad: i32,
}

impl Header {
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        Self {
            screen: _screen,
            result: Result::new(),
            color: false,
            diff: 0,
            interval: ::DEFAULT_INTERVAL,
            command: "".to_string(),
            output: ::IS_OUTPUT,
            active_pad: ::IS_WATCH_PAD,
        }
    }

    // 1st line
    fn printout_1st_line(&mut self, max_x: i32) {
        // interval second num to string
        // let interval = format!("{:.*}", 2, self.result.interval);
        let interval = format!("{:.2}", self.interval);

        // print interval and exec command
        // ex) Every XXs: Command...
        wmove(self.screen, 0, 0);
        waddstr(
            self.screen,
            &format!("Every {}s: {}", interval, self.result.clone().command),
        );

        // mvprintw(
        //     0,
        //     0,
        //     &format!("Every {}s: {:}", interval, self.result.clone().command),
        // );

        // print Now time in right end
        // ex) YYYY-mm-dd HH:MM:SS
        mvprintw(0, max_x - 20, &format!("{}", self.result.clone().timestamp));
    }

    // 2nd line
    fn printout_2nd_line(&mut self, max_x: i32) {
        // set var
        let _color_length = 13; // "Color: "(7) + "False"(5) + 1
        let _output_length = 15; // "Output: "(8) + "output"(6) + 1
        let _pad_length = 16; // "Active: "(8) + "history"(7) + 1
        let _diff_length = 12; // "Diff: "(6) + "Watch"(5) + 1

        // print color mode
        match self.color {
            true => {
                attron(COLOR_PAIR(COLORSET_B_D) | A_BOLD());
                mvprintw(
                    1,
                    max_x - (_color_length + _output_length + _pad_length + _diff_length),
                    &format!("Color: True "),
                );
                attroff(COLOR_PAIR(COLORSET_B_D) | A_BOLD());
            }
            false => {
                attron(COLOR_PAIR(COLORSET_D_D) | A_BOLD());
                mvprintw(
                    1,
                    max_x - (_color_length + _output_length + _pad_length + _diff_length),
                    &format!("Color: False"),
                );
                attroff(COLOR_PAIR(COLORSET_D_D) | A_BOLD());
            }
        }

        // print output type
        let mut _output_type = "";
        match self.output {
            ::IS_OUTPUT => _output_type = "output",
            ::IS_STDOUT => _output_type = "stdout",
            ::IS_STDERR => _output_type = "stderr",
            _ => (),
        }
        attron(COLOR_PAIR(COLORSET_Y_D));
        mvprintw(
            1,
            max_x - (_output_length + _pad_length + _diff_length),
            &format!("Output: {}", _output_type),
        );
        attroff(COLOR_PAIR(COLORSET_Y_D));

        // print pad
        let mut _active_type = "";
        match self.active_pad {
            ::IS_WATCH_PAD => _active_type = "watch  ",
            ::IS_HISTORY_PAD => _active_type = "history",
            _ => (),
        };
        attron(COLOR_PAIR(COLORSET_C_D));
        mvprintw(
            1,
            max_x - (_pad_length + _diff_length),
            &format!("Active: {}", _active_type),
        );
        attroff(COLOR_PAIR(COLORSET_C_D));

        // print diff
        let mut _diff_type = "";
        match self.diff {
            0 => _diff_type = "None",
            1 => _diff_type = "Watch",
            2 => _diff_type = "Line",
            3 => _diff_type = "Word",
            _ => (),
        };
        attron(COLOR_PAIR(COLORSET_M_D));
        mvprintw(1, max_x - _diff_length, &format!("Diff: {}", _diff_type));
        attroff(COLOR_PAIR(COLORSET_M_D));

        // print Now selected history num
    }

    // update header
    pub fn update(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        // update 1st line
        if self.result.clone().status {
            attron(COLOR_PAIR(COLORSET_G_D));
            self.printout_1st_line(max_x);
            attroff(COLOR_PAIR(COLORSET_G_D));
        } else {
            attron(COLOR_PAIR(COLORSET_R_D));
            self.printout_1st_line(max_x);
            attroff(COLOR_PAIR(COLORSET_R_D));
        }

        // update 2nd line
        self.printout_2nd_line(max_x);

        refresh();
    }
}
