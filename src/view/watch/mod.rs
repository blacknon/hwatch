mod diff;
mod window;

use std::sync::Mutex;
use ncurses::*;

use cmd::Result;
use self::window::WatchPad;
use view::*;

pub struct Watch {
  pub diff: i32,
  pub output_type: i32,

  pub count: i32,
  pub latest_result: Result,
  pub before_result: Result,

  pub watchpad: self::window::WatchPad,

  pub history: Mutex<Vec<Result>>,
  pub history_pad: WINDOW,
  pub history_pad_width: i32,
  pub history_pad_lines: i32,
  pub history_pad_position: i32,

  pub selected_position: i32, // history select position

  pub screen: WINDOW,
}

impl Watch {
  // set default value
  pub fn new(_screen: WINDOW, _diff: i32, _historywidth: i32) -> Self {
    let _watch = WatchPad::new(_screen.clone());
    Self {
      diff: _diff,
      output_type: IS_OUTPUT,

      count: 0,
      latest_result: Result::new(),
      before_result: Result::new(),

      watchpad: _watch,

      history: Mutex::new(vec![]),
      history_pad: newpad(0, 0),
      history_pad_width: _historywidth,
      history_pad_lines: 0,
      history_pad_position: 0,

      selected_position: 0,

      screen: _screen,
    }
  }

  pub fn get_latest_history(&mut self) -> Result {
    let mut _result = Result::new();

    let mut _history = self.history.lock().unwrap();
    let _length = _history.len();
    if _length >= 1 {
      _result = _history[0].clone();
    }
    return _result;
  }

  pub fn draw_history_pad(&mut self) {
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(self.screen, &mut max_y, &mut max_x);
    refresh();

    // Create history_pad
    self.history_pad_lines = self.count.clone() + 1;
    self.history_pad = newpad(self.history_pad_lines, max_x);

    // print latest
    let _latest_status = self.latest_result.status;
    self.print_history(0, "latest             ".to_string(), _latest_status);

    // print history
    let mut _history = self.history.lock().unwrap().clone();

    let mut i = 1;
    let _length = _history.len();
    for x in 0.._length {
      let _timestamp = _history[x].timestamp.clone();
      let _status = _history[x].status.clone();

      self.print_history(i, _timestamp, _status);
      i += 1;
    }
    let history_pad_lastline = self.history_pad_position + max_y - 4;
    if self.selected_position >= history_pad_lastline {
      self.history_pad_position = self.selected_position - max_y + 3;
    }

    prefresh(
      self.history_pad,
      self.history_pad_position,
      0,
      2,
      max_x - self.history_pad_width,
      max_y - 1,
      max_x - 1,
    );
  }

  fn print_history(&mut self, position: i32, word: String, status: bool) {
    if position == self.selected_position {
      if status == true {
        // selected line and status true
        wattron(self.history_pad, A_REVERSE() | COLOR_PAIR(2));
        wprintw(self.history_pad, &format!(">{}\n", word));
        wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(2));
      } else {
        // selected line and status false
        wattron(self.history_pad, A_REVERSE() | COLOR_PAIR(3));
        wprintw(self.history_pad, &format!(">{}\n", word));

        wattroff(self.history_pad, A_REVERSE() | COLOR_PAIR(3));
      }
    } else {
      if status == true {
        // not selected line and status true
        wattron(self.history_pad, COLOR_PAIR(2));
        wprintw(self.history_pad, &format!(" {}\n", word));
        wattroff(self.history_pad, COLOR_PAIR(2));
      } else {
        // not selected line and status false
        wattron(self.history_pad, COLOR_PAIR(3));
        wprintw(self.history_pad, &format!(" {}\n", word));
        wattroff(self.history_pad, COLOR_PAIR(3));
      }
    }
  }

  pub fn history_scroll_up(&mut self) {
    if self.selected_position > 0 {
      self.selected_position -= 1;
      self.draw_history_pad();
      self.watch_update();
    }
  }

  pub fn history_scroll_down(&mut self) {
    if self.count > self.selected_position {
      self.selected_position += 1;
      self.draw_history_pad();
      self.watch_update();
    }
  }

  pub fn window_scroll_up(&mut self) {
    self.watchpad.scroll_up()
  }

  pub fn window_scroll_down(&mut self) {
    self.watchpad.scroll_down();
  }

  pub fn resize(&mut self) {
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(self.screen, &mut max_y, &mut max_x);

    resizeterm(max_y, max_x);

    self.draw_history_pad();
    self.watchpad.resize();
  }

  pub fn append_history(&mut self, _result: Result) {
    let mut history = self.history.lock().unwrap();
    history.insert(0, _result);
    self.count += 1;
  }

  pub fn watch_update(&mut self) {
    let count = self.count.clone();

    if self.diff != 0 && count > 1 {
      self.diff_watch_update();
    } else {
      self.plane_watch_update();
    }

    self.watchpad.draw_output_pad();
    self.draw_history_pad();
  }

  fn plane_watch_update(&mut self) {
    let target_result = self.get_target_result(0);

    self.watchpad.result = target_result.clone();
    self.watchpad.before_update_output_pad(self.output_type);
    self.watchpad.update_output_pad_text(self.diff, self.output_type);
  }

  fn diff_watch_update(&mut self) {
    let before_result = self.get_target_result(-1);
    let target_result = self.get_target_result(0);

    if target_result.output != before_result.output && self.selected_position != self.count {
      self.watchpad.result = target_result.clone();
      match self.diff {
        1 => self.watch_diff_print(before_result, target_result),
        2 => self.line_diff_print(before_result, target_result),
        _ => self.plane_watch_update(),
      }
    } else {
      self.watchpad.result = target_result.clone();
      self.watchpad.before_update_output_pad(self.output_type);
      self.watchpad.update_output_pad_text(self.diff, self.output_type);
    }
  }

  fn watch_diff_print(&mut self, before_result: Result, target_result: Result) {
    let mut _before_result_data = before_result.output.clone();
    let mut _target_result_data = target_result.output.clone();

    match self.output_type {
      IS_OUTPUT => {
        _before_result_data = before_result.output.clone();
        _target_result_data = target_result.output.clone();
      },
      IS_STDOUT => {
        _before_result_data = before_result.stdout.clone();
        _target_result_data = target_result.stdout.clone();
      },
      IS_STDERR => {
        _before_result_data = before_result.stderr.clone();
        _target_result_data = target_result.stderr.clone();
      },
      _ => {},
    };

    self.watchpad.before_update_output_pad(self.output_type);
    diff::watch_diff(
      self.watchpad.clone(),
      _before_result_data,
      _target_result_data,
    );
  }

  fn line_diff_print(&mut self, before_result: Result, target_result: Result) {
    let mut _before_result_data = before_result.output.clone();
    let mut _target_result_data = target_result.output.clone();

    match self.output_type {
      IS_OUTPUT => {
        _before_result_data = before_result.output.clone();
        _target_result_data = target_result.output.clone();
      },
      IS_STDOUT => {
        _before_result_data = before_result.stdout.clone();
        _target_result_data = target_result.stdout.clone();
      },
      IS_STDERR => {
        _before_result_data = before_result.stderr.clone();
        _target_result_data = target_result.stderr.clone();
      },
      _ => {},
    };


    let line_diff_str = diff::line_diff_str_get(_before_result_data.clone(), _target_result_data.clone());
    self.watchpad.result_diff_output = line_diff_str;
    self.watchpad.before_update_output_pad(self.output_type);
    diff::line_diff(
      self.watchpad.clone(),
      _before_result_data,
      _target_result_data,
    );
    self.watchpad.result_diff_output = String::new();
  }

  // @note:
  //    [get_type value]
  //    0 ... target result
  //   -1 ... before result
  //    1 ... next result
  fn get_target_result(&mut self, get_type: i32) -> Result {
    let mut result = Result::new();
    if self.selected_position != 0 {
      let mut _history = self.history.lock().unwrap().clone();
      let _length = _history.len();

      let mut i = 1;
      for x in 0.._length {
        if get_type == 0 && i == self.selected_position {
          result = _history[x].clone();
        } else if get_type == -1 && i == self.selected_position + 1 {
          result = _history[x].clone();
        } else if get_type == 1 && i == self.selected_position - 1 {
          result = _history[x].clone();
        }
        i += 1;
      }
    } else {
      if get_type == 0 {
        result = self.latest_result.clone();
      } else if get_type == -1 {
        result = self.before_result.clone();
      }
    }
    return result;
  }

  pub fn mouse_action(&mut self,_mevent: MEVENT) {
    let _mouse_event = _mevent.bstate as i32;
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(self.screen, &mut max_y, &mut max_x);

    // mouse is not on header
    if _mevent.y > 1 {
      match _mouse_event {
        // mouse left button click
        BUTTON1_CLICKED => {
          if max_x - self.history_pad_width < _mevent.x {
            let _mouse_select_line = _mevent.y - 2 + self.history_pad_position;

            if self.history_pad_lines > _mouse_select_line {
              self.selected_position = _mouse_select_line;

              // update draw
              self.draw_history_pad();
              self.watch_update();
            }
          }
        },

        // mouse wheel up
        BUTTON4_PRESSED => {
          if max_x - self.history_pad_width < _mevent.x {
            // mouse on history
            self.history_scroll_up();
          } else {
            // mouse on watch
            self.window_scroll_up();
          }
        },

        // mouse wheel down
        BUTTON5_PRESSED => {
          // mouse on history
          if max_x - self.history_pad_width < _mevent.x {
            self.history_scroll_down();
          } else {
            // mouse on watch
            self.window_scroll_down();
          }
        }
        _ => {},
      }
    }
  }

  // fn get_result_output(&mut self,_result: Result) ->String {
  //     let _result_output =  String::new();
  //     return _result_output
  // }

  pub fn exit(&mut self) {
    self.watchpad.exit();
    delwin(self.history_pad);
  }
}
