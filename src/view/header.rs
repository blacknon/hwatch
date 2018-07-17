extern crate ncurses;

use ncurses::*;

use cmd::Result;

pub struct Header {
    pub screen: ncurses::WINDOW,
    pub result: Result,
    pub diff: i32,
}

impl Header {
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        Self{
            screen: _screen,
            result: Result::new(),
            diff: 0,
        }
    }

    fn print_1st_header(&mut self,max_x: i32) {
        let interval_string = format!("{:.*}", 2, self.result.interval);

        mvprintw(0, 0, &format!("Every {}s: {}", interval_string, self.result.clone().command));
        mvprintw(0, max_x - 20, &format!("{}", self.result.clone().timestamp));
    }

    fn print_2nd_header(&mut self,max_x: i32) {
        attron(COLOR_PAIR(4));
        match self.diff {
            0 => mvprintw(1, max_x - 12, &format!("{}", "diff: None") ),
            1 => mvprintw(1, max_x - 12, &format!("{}", "diff: Watch") ),
            2 => mvprintw(1, max_x - 12, &format!("{}", "diff: Line") ),
            3 => mvprintw(1, max_x - 12, &format!("{}", "diff: Word") ),
            _ => mvprintw(1, max_x - 12, &format!("{}", "diff: None") ),
        };
        attroff(COLOR_PAIR(4));
    }

    pub fn update_header(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        if self.result.clone().status {
            attron(COLOR_PAIR(2));
            self.print_1st_header(max_x);
            attroff(COLOR_PAIR(2));
        } else {
            attron(COLOR_PAIR(3));
            self.print_1st_header(max_x);
            attroff(COLOR_PAIR(3));
        }
        self.print_2nd_header(max_x);
        refresh();
    }
}