extern crate ncurses;

use ncurses::*;

use cmd::Result;

pub struct Header {
    pub screen: ncurses::WINDOW
}

impl Header {
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        Self{
            screen: _screen
        }
    }

    pub fn update_header(&mut self,_result: Result){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        let interval_string = format!("{:.*}", 2, _result.interval);

        if _result.status {
            attron(COLOR_PAIR(2));
            mvprintw(0, 0, &format!("Every {}s: {}", interval_string, _result.command));
            mvprintw(0, max_x - 20, &format!("{}", _result.timestamp));
            attroff(COLOR_PAIR(2));
        } else {
            attron(COLOR_PAIR(3));
            mvprintw(0, 0, &format!("Every {}s: {}", interval_string, _result.command));
            mvprintw(0, max_x - 20, &format!("{}", _result.timestamp));
            attroff(COLOR_PAIR(3));
        }
        refresh();
    }
}