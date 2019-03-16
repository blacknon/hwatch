extern crate ncurses;

use ncurses::*;
use view::*;

use cmd::Result;

pub struct Header {
  pub screen: ncurses::WINDOW,
  pub result: Result,
  pub diff: i32,
  pub active_pad: i32,
}

impl Header {
  pub fn new(_screen: ncurses::WINDOW) -> Self {
    Self {
      screen: _screen,
      result: Result::new(),
      diff: 0,
      active_pad: IS_WATCH_PAD,
    }
  }

  fn print_1st_header(&mut self, max_x: i32) {
    let interval_string = format!("{:.*}", 2, self.result.interval);

    mvprintw(
      0,
      0,
      &format!(
        "Every {}s: {}",
        interval_string,
        self.result.clone().command
      ),
    );
    mvprintw(0, max_x - 20, &format!("{}", self.result.clone().timestamp));
  }

  fn print_2nd_header(&mut self, max_x: i32) {
    let mut _active_type = "";
    match self.active_pad {
      IS_WATCH_PAD => _active_type = "watch  ",
      IS_HISTORY_PAD => _active_type = "history",
      _ => (),
    };
    attron(COLOR_PAIR(5));
    mvprintw(1, max_x - 28, &format!("Active: {}", _active_type));
    attroff(COLOR_PAIR(5));

    let mut _diff_type = "";
    match self.diff {
      0 => _diff_type = "None",
      1 => _diff_type = "Watch",
      2 => _diff_type = "Line",
      3 => _diff_type = "Word",
      _ => (),
    };
    attron(COLOR_PAIR(4));
    mvprintw(1, max_x - 12, &format!("Diff: {}", _diff_type));
    attroff(COLOR_PAIR(4));
  }

  pub fn update_header(&mut self) {
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
