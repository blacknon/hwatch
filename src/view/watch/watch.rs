// @TODO
//     こちらのファイルではwatchpadの表示機能だけ制御させるよう、書き換えを行う！
//     Structにresultいるか？？いらないんじゃないか？？

// module
use ncurses::*;

#[derive(Clone)]
pub struct WatchPad {
    pub screen: WINDOW,
    pub pad: WINDOW,
    pub pad_lines: i32,
    pub pad_position: i32,
}

impl WatchPad {
    // set default value
    pub fn new(_screen: WINDOW) -> Self {
        Self {
            screen: _screen,
            pad: newpad(0, 0),
            pad_lines: 0,
            pad_position: 0,
        }
    }

    // print data
    pub fn print(&mut self, _data: String, _front_color: i16, _back_color: i16, _flags: Vec<i32>) {
        // set flags
        for flag in &_flags {
            match flag {
                1 => wattron(self.pad, A_BOLD()),
                7 => wattron(self.pad, A_REVERSE()),
                _ => wattron(self.pad, A_BOLD()),
            };
        }

        // create color set
        init_pair(0, _front_color, _back_color);

        // set color
        wattron(self.pad, COLOR_PAIR(0));

        // print data
        wprintw(self.pad, &format!("{}", _data));

        // unset color
        wattron(self.pad, COLOR_PAIR(0));

        // unset flags
        for flag in &_flags {
            match flag {
                1 => wattroff(self.pad, A_BOLD()),
                7 => wattroff(self.pad, A_REVERSE()),
                _ => wattroff(self.pad, A_BOLD()),
            };
        }
    }

    // printに統合して削除する
    pub fn print_watch_data(&mut self, _char: String, _reverse: bool, _color_code: i16) {
        if _reverse {
            wattron(self.pad, A_REVERSE());
            self.print_char_to_color_pair(_char, _color_code);
            wattroff(self.pad, A_REVERSE());
        } else {
            self.print_char_to_color_pair(_char, _color_code);
        }
    }

    fn print_char_to_color_pair(&mut self, _char: String, _color_code: i16) {
        if _color_code != 0 {
            wattron(self.pad, COLOR_PAIR(_color_code));
            wprintw(self.pad, &format!("{}", _char));
            wattroff(self.pad, COLOR_PAIR(_color_code));
        } else {
            wprintw(self.pad, &format!("{}", _char));
        }
    }

    pub fn draw_output_pad(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        self.prefresh(max_y, max_x);
    }

    pub fn scroll_up(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        if self.pad_lines > max_y && self.pad_position > 0 {
            self.pad_position -= 1;
            self.prefresh(max_y, max_x);
        }
    }

    pub fn scroll_down(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        if self.pad_lines > max_y && self.pad_position < (self.pad_lines - max_y + 2 - 1) {
            self.pad_position += 1;
            self.prefresh(max_y, max_x);
        }
    }

    pub fn resize(&mut self) {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        resizeterm(max_y, max_x);
        self.prefresh(max_y, max_x);
    }

    fn prefresh(&mut self, max_y: i32, max_x: i32) {
        prefresh(
            self.pad,
            self.pad_position,
            0,
            2,
            0,
            max_y - 1,
            max_x - (::HISTORY_WIDTH + 2),
        );
    }

    pub fn exit(&self) {
        endwin();
    }
}

// @TODO:
//    下のコードを参考に、ANSIカラーコードからNcurses向けのカラーコードへの変換処理を実装する
//    example)
//    https://github.com/viseztrance/flow/blob/f34f34210f9bfcded8ae6c6740ab2f2fe2aa28c9/src/utils/ansi_decoder.rs
// @Note:
//    この関数内でANSI Colorとその出力結果の配列にして、それを返すようにする。
//    処理としては、最初にこの関数を実行してANSI Colorとその出力結果で配列化して、それをベースにwatchの各処理をさせるように記述すればいけるか？？？
// fn get_ansi_array() {}

// つまり、print時に最初にANSIのカラーコード単位で出力内容と配列を出して、それをforで今までの出力用関数にわたしてやるときれいかも？？？
//
//
//
//
//
