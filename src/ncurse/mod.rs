extern "C" {
    pub fn setlocale(category: i32, locale: *const u8) -> *const u8;
}

extern crate ncurses;

use self::ncurses::*;

pub struct View {
    pub timestamp: String,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub status: bool,
    pub mode: bool,
    pub position: i32,
    pub window: self::ncurses::WINDOW,
    pub pad: self::ncurses::WINDOW,
}

impl View {
    pub fn new() -> Self {
        Self {
            timestamp: "".to_string(),
            command: "".to_string(),
            stdout: "".to_string(),
            stderr: "".to_string(),
            status: true,
            mode: true,
            position: 0,
            window: initscr(),
            pad: newpad(0,0),
        }
    }

    pub fn view_watch_screen(&mut self) {
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
    
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(_win, &mut max_y, &mut max_x);
        // getmaxyx(_win, &mut max_y, &mut max_x);
    
        // count pad lines
        let mut _pad_lines = 0;
        for _output_line in self.stdout.split("\n") {
            _pad_lines += get_pad_lines(_output_line.to_string(),max_x);
        }
    
        // Create pad
        // let _pad = newpad(_pad_lines, self.max_x);
        self.pad = newpad(_pad_lines, max_x);
        refresh();
    
        // print pad
        for _output_line in self.stdout.split("\n") {
            wprintw(self.pad, &format!("{}\n", _output_line));
        }
    
        // print first 2 lines
        init_pair(1, COLOR_GREEN, -1);
        init_pair(2, COLOR_RED, -1);
        if self.status {
            // let attr = attron(A_BOLD() | A_BLINK() | COLOR_PAIR(1));
            attron(COLOR_PAIR(1));
            mvprintw(0, 0, &format!("{}", self.timestamp));
            mvprintw(1, 0, &format!("{}", self.command));
            attroff(COLOR_PAIR(1));
        } else {
            attron(COLOR_PAIR(2));
            mvprintw(0, 0, &format!("{}", self.timestamp));
            mvprintw(1, 0, &format!("{}", self.command));
            attroff(COLOR_PAIR(2));
        }
        
        // view pub
        refresh();
        prefresh(self.pad, self.position, 0, 2, 0, max_y - 1, max_x - 1);
    
        // // let mut _position = self.position;
        // loop {
        //     let _input = getch();
        //     if _input == KEY_F1 {
        //         break;
        //     }
    
        //     if _input == KEY_DOWN {
        //         if self.position < _pad_lines - max_y - 1 + 2 {
        //             self.position += 1;
        //             prefresh(_pad, self.position, 0, 2, 0, max_y - 1, max_x - 1);
        //         }
        //     }
    
        //     if _input == KEY_UP {
        //         if self.position > 0 {
        //             self.position -= 1;
        //             prefresh(_pad, self.position, 0, 2, 0, max_y - 1, max_x - 1);
        //         }
        //     }
        // }
        //endwin();
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
