extern crate ncurses;

use self::ncurses::*;

// pub struct Watch {
//     pub stdout: String,
//     pub stderr: String,
//     pub position: i32
// }

pub fn print_ncurse_screen(_output: String) {
    let _output_bytes = _output.bytes();

    // start ncurse
    let win = initscr();
    noecho();
    cbreak();
    scrollok(win, true);

    
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(win, &mut max_y, &mut max_x);

    let mut _lineno = 1;

    for ch in _output_bytes{
        let mut cur_x = 0;
        let mut cur_y = 0;
        getyx(win, &mut cur_y, &mut cur_x);
        // println!("{:?},{:?}", cur_y,max_y);

        if cur_y >= _lineno +1 {
            _lineno += 1;
        }
        
        if max_y <= _lineno +1 {
            getch();
            wscrl(win, 1);
            wmove(win, max_x-1, 0);
            _lineno -=2;
            //break;
        } else {
            waddch(win, ch.into());
            //getch();
        }


        
        refresh();
    }

        // loop {
        //     let _input = getch();
        //     if _input.to_string() == "q" {
        //         break;
        //     }
        //     wscrl(win, 1);
        //     wmove(win, max_x-1, 0);
        //     // while ((ch = fgetc(fp)) != EOF) {
        //     //     if (ch == '\n') {
        //     //         lineno++;
        //     //         break;
        //     //     }
        //     //     waddch(win, ch);
        //     // }
        //     // refresh();
        //     }
        // }
    // refresh();
    // println!("{:?},{:?}", max_x,max_y);

    endwin();
    println!("{:?},{:?}", max_x,max_y);
    // println!("{:?}", _x);

    }