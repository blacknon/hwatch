use ncurses::*;
use cmd::Result;

#[derive(Clone)]
pub struct WatchPad {
    pub result: Result,

    pub screen: WINDOW,
    pub pad: WINDOW,
    pub pad_lines: i32,
    pub pad_position: i32,
}


impl WatchPad {
    // set default value
    pub fn new(_screen: WINDOW) -> Self {
        Self {
            result: Result::new(),

            screen: _screen,
            pad: newpad(0,0),
            pad_lines: 0,
            pad_position: 0,
        }
    }

    pub fn before_update_output_pad(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        let mut _pad_lines = 0;
        for _output_line in self.result.output.clone().split("\n") {
            // self.pad_lines += 3
            _pad_lines += get_pad_lines(_output_line.to_string(), max_x -23);
        }

        self.pad_lines = _pad_lines;
        self.pad = newpad(self.pad_lines.clone(), max_x - 23);
    }

    pub fn update_output_pad_text(&mut self) {
        wprintw(self.pad, &format!("{}", self.result.output.clone()));
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
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 23);
    }

    pub fn scroll_up(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        if self.pad_lines > max_y && self.pad_position > 0 {
            self.pad_position -= 1;
            prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 23);
        }
    }

    pub fn scroll_down(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        if self.pad_lines > max_y && self.pad_position < (self.pad_lines - max_y + 2 - 1) {
            self.pad_position += 1;
            prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 23);
        }
    }

    pub fn resize(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);
        resizeterm(max_y,max_x);
        prefresh(self.pad, self.pad_position, 0, 2, 0, max_y - 1, max_x - 23);
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
