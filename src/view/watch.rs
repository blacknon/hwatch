extern "C" {
    pub fn setlocale(category: i32, locale: *const u8) -> *const u8;
}
extern crate ncurses;

use self::ncurses::*;
use cmd::Result;

pub struct Watch {
    pub result: Result,
    pub mode: bool,
    pub position: i32,
    pub window: self::ncurses::WINDOW,
    pub pad: self::ncurses::WINDOW,
    pub pad_lines: i32
}


impl Watch{
    // set default value
    pub fn new() -> Self {
        let _result = Result::new();

        Self {
            result: _result,
            mode: true,
            position: 0,
            window: initscr(),
            pad: newpad(0,0),
            pad_lines: 0
        }
    }

    // init ncurses
    pub fn init(&mut self) {
        unsafe {
            setlocale(0 /* = LC_CTYPE */, "".as_ptr());
        }
        // Start ncurses
        let _win = self.window;
        start_color();
        use_default_colors();
        cbreak();
        keypad(_win, true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);       

        init_pair(1, COLOR_GREEN, -1);
        init_pair(2, COLOR_RED, -1);
    }

    pub fn update_output_pad(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.window, &mut max_y, &mut max_x);

        let mut _pad_lines = 0;
        for _output_line in self.result.output.split("\n") {
            _pad_lines += get_pad_lines(_output_line.to_string(),max_x);
        }

        self.pad_lines = _pad_lines;
        self.pad = newpad(self.pad_lines, max_x);
        refresh();

        //output pad
        wprintw(self.pad, &format!("{}", self.result.output));
    }

    pub fn draw_output_pad(&mut self) {
        if self.result.status {
            attron(COLOR_PAIR(1));
            mvprintw(0, 0, &format!("{}", self.result.timestamp));
            mvprintw(1, 0, &format!("{}", self.result.command));
            attroff(COLOR_PAIR(1));
        } else {
            attron(COLOR_PAIR(2));
            mvprintw(0, 0, &format!("{}", self.result.timestamp));
            mvprintw(1, 0, &format!("{}", self.result.command));
            attroff(COLOR_PAIR(2));
        }
        refresh();

        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.window, &mut max_y, &mut max_x);
        prefresh(self.pad, self.position, 0, 2, 0, max_y - 1, max_x - 1);
    }

    pub fn update(&mut self,_result: Result){
        self.result.timestamp = _result.timestamp;
        self.result.command = _result.command;
        self.result.status = _result.status;
        self.result.output = _result.output;
        self.result.stdout = _result.stdout;
        self.result.stderr = _result.stderr;

        self.update_output_pad();
        self.draw_output_pad();
    }

    pub fn scroll_up(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.window, &mut max_y, &mut max_x);
        if self.position > 0 {
            self.position -= 1;
            prefresh(self.pad, self.position, 0, 2, 0, max_y - 1, max_x - 1);
        }
    }

    pub fn scroll_down(&mut self){
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.window, &mut max_y, &mut max_x);
        if self.position < self.pad_lines - max_y - 1 + 2 {
            self.position += 1;
            prefresh(self.pad, self.position, 0, 2, 0, max_y - 1, max_x - 1);
        }
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
