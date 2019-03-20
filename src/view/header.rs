extern crate ncurses;

use ncurses::*;

use cmd::Result;

pub struct Header {
    pub screen: ncurses::WINDOW,
    pub result: Result,
    pub diff: i32,
    pub output: i32,
    pub active_pad: i32,
}

impl Header {
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        Self {
            screen: _screen,
            result: Result::new(),
            diff: 0,
            output: ::IS_OUTPUT,
            active_pad: ::IS_WATCH_PAD,
        }
    }

    // 1st line
    fn printout_1st_line(&mut self, max_x: i32) {
        let interval_string = format!("{:.*}", 2, self.result.interval);

        // print interval and exec command
        // ex) Every XXs: Command...
        mvprintw(
            0,
            0,
            &format!(
                "Every {}s: {}",
                interval_string,
                self.result.clone().command
            ),
        );

        // print Now time in right end
        // ex) YYYY-mm-dd HH:MM:SS
        mvprintw(0, max_x - 20, &format!("{}", self.result.clone().timestamp));
    }

    // 2nd line
    fn printout_2nd_line(&mut self, max_x: i32) {
        let mut _output_type = "";
        match self.output {
            ::IS_OUTPUT => _output_type = "output",
            ::IS_STDOUT => _output_type = "stdout",
            ::IS_STDERR => _output_type = "stderr",
            _ => (),
        }
        attron(COLOR_PAIR(6));
        mvprintw(1, max_x - 43, &format!("Output: {}", _output_type));
        attroff(COLOR_PAIR(6));

        let mut _active_type = "";
        match self.active_pad {
            ::IS_WATCH_PAD => _active_type = "watch  ",
            ::IS_HISTORY_PAD => _active_type = "history",
            _ => (),
        };
        attron(COLOR_PAIR(5));
        mvprintw(1, max_x - 28, &format!("Active: {}", _active_type));
        attroff(COLOR_PAIR(5));

        let mut _diff_type = "";
        match self.diff {
            0 => _diff_type = "None",
            1 => _diff_type = "Watch",
            2 => _diff_type = "Line",
            3 => _diff_type = "Word",
            _ => (),
        };
        attron(COLOR_PAIR(4));
        mvprintw(1, max_x - 12, &format!("Diff: {}", _diff_type));
        attroff(COLOR_PAIR(4));
    }

    // update header
    pub fn update(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        // update 1st line
        if self.result.clone().status {
            attron(COLOR_PAIR(2));
            self.printout_1st_line(max_x);
            attroff(COLOR_PAIR(2));
        } else {
            attron(COLOR_PAIR(3));
            self.printout_1st_line(max_x);
            attroff(COLOR_PAIR(3));
        }

        // update 2nd line
        self.printout_2nd_line(max_x);

        refresh();
    }
}
