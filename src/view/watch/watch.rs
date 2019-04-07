// module
use ncurses::*;
use view::color::*;

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
                &IS_BOLD => wattron(self.pad, A_BOLD()),
                &IS_UNDERLINE => wattron(self.pad, A_UNDERLINE()),
                &IS_BLINK => wattron(self.pad, A_BLINK()),
                &IS_REVERSE => wattron(self.pad, A_REVERSE()),
                _ => wattroff(self.pad, A_NORMAL()), // Error avoidance
            };
        }

        let colorset_string = format!("{}{}", _front_color.to_string(), _back_color.to_string());
        let mut colorset: i16 = colorset_string.parse::<i16>().unwrap();;

        if colorset < 0 {
            colorset = 0;
        }

        // set color
        wattron(self.pad, COLOR_PAIR(colorset));

        // print data
        wprintw(self.pad, &format!("{}", _data));

        // unset color
        wattron(self.pad, COLOR_PAIR(colorset));

        // unset flags
        for flag in &_flags {
            match flag {
                &IS_BOLD => wattroff(self.pad, A_BOLD()),
                &IS_UNDERLINE => wattroff(self.pad, A_UNDERLINE()),
                &IS_BLINK => wattroff(self.pad, A_BLINK()),
                &IS_REVERSE => wattroff(self.pad, A_REVERSE()),
                _ => wattroff(self.pad, A_NORMAL()), // Error avoidance
            };
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
