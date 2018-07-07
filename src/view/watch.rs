extern crate ncurses;

use ncurses::*;
use cmd::Result;

#[derive(Clone)]
pub struct Watch {
    pub diff: bool,
    pub result: Result,
    pub pad_position: i32,
    pub screen: ncurses::WINDOW,
    pub pad: self::ncurses::WINDOW,
    pub pad_lines: i32
}


impl Watch {
    // set default value
    pub fn new(_screen: ncurses::WINDOW) -> Self {
        let _result = Result::new();

        Self {
            diff: false,
            result: _result,
            pad_position: 0,
            screen: _screen,
            pad: newpad(0,0),
            pad_lines: 0,
        }
    }

    pub fn before_update_output_pad(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        let mut _pad_lines = 0;
        for _output_line in self.result.output.split("\n") {
            _pad_lines += get_pad_lines(_output_line.to_string(),max_x);
        }

        self.pad_lines = _pad_lines;
        self.pad = newpad(self.pad_lines, max_x);
        refresh();
    }

    pub fn update_output_pad_text(&mut self) {
        //output pad
        wprintw(self.pad, &format!("{}", self.result.output));
    }

    pub fn update_output_pad_char(&mut self, _char: String, _reverse: bool) {
        if _reverse {
            wattron(self.pad,A_REVERSE());
            wprintw(self.pad, &format!("{}", _char));
            wattroff(self.pad,A_REVERSE());
        } else {
            wprintw(self.pad, &format!("{}", _char));
        }
    }

    pub fn draw_output_pad(&mut self) {
        if self.result.status {
            attron(COLOR_PAIR(2));
            mvprintw(0, 0, &format!("{}", self.result.timestamp));
            mvprintw(1, 0, &format!("{}", self.result.command));
            attroff(COLOR_PAIR(2));
        } else {
            attron(COLOR_PAIR(3));
            mvprintw(0, 0, &format!("{}", self.result.timestamp));
            mvprintw(1, 0, &format!("{}", self.result.command));
            attroff(COLOR_PAIR(3));
        }
        refresh();

        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        
        prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 22);
    }

    pub fn update(&mut self,_result: Result){
        self.result = _result;

        self.before_update_output_pad();
        self.update_output_pad_text();
        self.draw_output_pad();
    }

    pub fn scroll_up(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        if self.pad_lines > max_y && self.pad_position > 0 {
            self.pad_position -= 1;
            prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 22);
        }
    }

    pub fn scroll_down(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        if self.pad_lines > max_y && self.pad_position < (self.pad_lines - max_y - 1 + 2) {
            self.pad_position += 1;
            prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 22);
        }
    }

    pub fn resize(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        resizeterm(max_y,max_x);
        prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 22);
    }

    pub fn exit(&self) {
        endwin();
    }
}


// get pad lines from string
fn get_pad_lines(_string: String, _width: i32) -> i32 {
    let char_vec: Vec<char> = _string.chars().collect();
    let mut _char_count = 0;
    let mut _line_count = 1;

    for ch in char_vec {
        if ch.to_string().len() > 1 {
            _char_count += 2;
        } else {
            _char_count += 1;
        }

        if _char_count == _width {
            _line_count += 1;
            _char_count = 0;
        }
    }
    return _line_count;
}
