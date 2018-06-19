extern "C" {
    pub fn setlocale(category: i32, locale: *const u8) -> *const u8;
}

extern crate ncurses;

use self::ncurses::*;

// pub struct View {
//     pub timestamp: String,
//     pub command: String,
//     pub stdout: String,
//     pub stderr: String,
//     pub position: i32
// }

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

pub fn print_ncurse_screen(_output: String) {
    unsafe {
        setlocale(0 /* = LC_CTYPE */, "".as_ptr());
    }
    // Start ncurses
    let _win = initscr();
    start_color();
    use_default_colors();
    cbreak();
    keypad(_win, true);
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(_win, &mut max_y, &mut max_x);

    // count pad lines
    let mut _pad_lines = 0;
    for _output_line in _output.split("\n") {
        _pad_lines += get_pad_lines(_output_line.to_string(),max_x);
    }

    // Create pad
    let _pad = newpad(_pad_lines, max_x);
    // let _pad = newpad(max_y, max_x);
    // let _subwin = subwin(win, max_y - 3, max_x - 2, 2, 1);
    // let _subpad = subpad(_pad,_pad_lines, max_x - 2,2,1);
    refresh();

    // print pad
    for _output_line in _output.split("\n") {
        // wprintw(_subpad, &format!("{}\n", _output_line));
        wprintw(_pad, &format!("{}\n", _output_line));
    }

    refresh();

    // 
    prefresh(_pad, 0, 0, 2, 0, max_y - 1, max_x - 1);
    // prefresh(_subpad, 0, 0, 0, 0, max_y - 5, max_x - 5);

    let mut _position = 0;
    loop {
        let _input = getch();
        if _input == KEY_F1 {
            break;
        }

        if _input == KEY_DOWN {
            if _position < _pad_lines - max_y - 1 + 2 {
                _position += 1;
                prefresh(_pad, _position, 0, 2, 0, max_y - 1, max_x - 1);
                // prefresh(pad, pmin_row, pmin_col, smin_row, smin_col, smax_row, smax_col)
                // prefresh(_subpad, _position, 0, 0, 0, max_y - 5, max_x - 5);
            }
        }

        if _input == KEY_UP {
            if _position > 0 {
                _position -= 1;
                prefresh(_pad, _position, 0, 2, 0, max_y - 1, max_x - 1);
                // prefresh(_subpad, _position, 0, 0, 0, max_y - 5, max_x - 5);

            }
        }
    }
    endwin();
    }