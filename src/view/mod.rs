// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): キーワード検索機能の追加(v0.1.7)
//     - `/`でキーボード入力モードに
//     - `f`でフィルターモードの切り替えか？(ハイライトモードとフィルタリングモード)
//     - ESCで元に戻す
//     - lessみたいに、キーワードの検索が行えるようにする
//     - 検索方式として、`ハイライト方式`及び`絞り込み方式`の2つが必要？

// TODO(blacknon): キー入力の追加
//     - PageUpでページアップ
//     - PageDownでページダウン

// TODO(blacknon): キー入力の変更機能を追加？(v0.1.7？)
//     - pecoのconfig的なやつ？何かしらファイルがあるだろうから探す

// module
use ncurses::*;
use std::env;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

// local module
mod color;
mod header;
mod watch;
use self::watch::Watch;
use cmd::Result;
use common::*;
use event::Event;
use view::color::*;

/// Struct at watch view window.
pub struct View {
    pub done: bool,
    pub screen: WINDOW,
    pub header: header::Header,
    pub watch: watch::Watch,
    pub logfile: String,
    cursor_mode: i32,
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}

/// Trail at watch view window.
impl View {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        //! method at create new view trail.

        // Set locale
        let locale_conf = LcCategory::all;
        let lang = get_lang();
        setlocale(locale_conf, &lang);

        // Create ncurses screen
        let _screen = initscr();

        cbreak();
        keypad(_screen, true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        // set color
        setup_colorset();

        let _watch = Watch::new(_screen.clone());
        Self {
            done: false,
            screen: _screen,
            header: header::Header::new(_screen.clone()),
            watch: _watch,
            logfile: "".to_string(),
            cursor_mode: ::CURSOR_NORMAL_WINDOW,
            tx: tx,
            rx: rx,
        }
    }

    fn exit(&mut self) {
        //! method at exit view trail.

        self.watch.exit();
        let _ = self.tx.send(Event::Exit);
    }

    fn update(&mut self, _result: Result) {
        //! update view watch and history window.

        // set header diff flag
        self.header.diff = self.watch.diff;

        // update latest result
        self.watch.latest_result = _result.clone();

        // history append result
        if self.watch.get_latest_history().output != _result.output {
            clear();
            self.header.result = _result.clone();
            self.header.update();

            // add selected positon
            if self.watch.selected != 0 {
                self.watch.selected += 1;
            }
            self.watch.append_history(_result.clone());
            self.watch.update();
            self.watch.draw_history();

            // logging data
            // TODO(blacknon): warning出てるので対応
            if self.logfile != "".to_string() {
                logging_result(&self.logfile, &_result);
            }
        } else {
            if self.watch.selected == 0 {
                // if history selected latest, update watch window.
                self.header.result = _result.clone();
                self.header.update();
                self.watch.update();
                self.watch.draw_history();
            } else {
                self.header.result = _result.clone();
                self.header.update();
            }
        }

        if self.cursor_mode == ::CURSOR_HELP_WINDOW {
            self.watch.draw_help()
        }

        drop(_result);
    }

    fn toggle_color(&mut self) {
        //! toggle ansi color mode.

        if self.watch.color {
            self.watch.color = false
        } else {
            self.watch.color = true
        }

        // update header status
        self.header.color = self.watch.color;
    }

    fn toggle_show_help(&mut self) {
        //! toggle show help window

        // check self.cursor_mode
        if self.cursor_mode == ::CURSOR_NORMAL_WINDOW {
            self.cursor_mode = ::CURSOR_HELP_WINDOW;

            // update
            self.header.update();
            self.watch.update();

            // Switching view help window
            self.watch.toggle_help_window();
        } else if self.cursor_mode == ::CURSOR_HELP_WINDOW {
            self.cursor_mode = ::CURSOR_NORMAL_WINDOW;

            // Switching view help window
            self.watch.toggle_help_window();

            // update
            self.header.update();
            self.watch.update();
        }
    }

    fn toggle_diff(&mut self) {
        //! toggle diff mode

        // add num
        let mut now_diff = self.watch.diff;
        now_diff += 1;

        self.switch_diff(now_diff % 3);
    }

    pub fn switch_diff(&mut self, _diff: i32) {
        //! switch diff mode.

        // set value
        self.watch.diff = _diff;
        self.header.diff = self.watch.diff;
    }

    fn toggle_pad(&mut self) {
        //! change active pad (history/watch)

        // add num
        let mut now_pad = self.header.active_pad;
        now_pad += 3;

        self.header.active_pad = now_pad % 2;
        self.header.update();
    }

    fn up(&mut self) {
        //! up key action

        match self.header.active_pad {
            ::IS_WATCH_PAD => self.watch.window_up(),
            ::IS_HISTORY_PAD => {
                if self.watch.selected > 0 {
                    clear();
                    self.header.update();
                    self.watch.history_up();
                }
            }
            _ => (),
        }
    }

    fn down(&mut self) {
        //! down key action
        match self.header.active_pad {
            ::IS_WATCH_PAD => self.watch.window_down(),
            ::IS_HISTORY_PAD => {
                if self.watch.count > self.watch.selected {
                    clear();
                    self.header.update();
                    self.watch.history_down()
                }
            }
            _ => (),
        }
    }

    // TODO: up()のコピペなので関数作ったら修正する
    fn page_up(&mut self) {
        //! pgup key action

        match self.header.active_pad {
            ::IS_WATCH_PAD => self.watch.window_up(),
            ::IS_HISTORY_PAD => {
                if self.watch.selected > 0 {
                    clear();
                    self.header.update();
                    self.watch.history_up();
                }
            }
            _ => (),
        }
    }

    // TODO: down()のコピペなので関数作ったら修正する
    fn page_down(&mut self) {
        //! pgdown key action

        match self.header.active_pad {
            ::IS_WATCH_PAD => self.watch.window_down(),
            ::IS_HISTORY_PAD => {
                if self.watch.count > self.watch.selected {
                    clear();
                    self.header.update();
                    self.watch.history_down()
                }
            }
            _ => (),
        }
    }

    fn set_output_type(&mut self, _output_type: i32) {
        //! Set output type(stdout/stderr/output(with)) at header and watch pad.

        // set value
        self.watch.output_type = _output_type;
        self.header.output = _output_type;
    }

    pub fn set_color(&mut self, _color: bool) {
        //! Set color at header and watch pad

        // set value
        self.watch.color = _color;
        self.header.color = _color;
    }

    pub fn set_command(&mut self, _command: String) {
        //! set command at header and watch pad

        self.header.command = _command;
    }

    pub fn set_interval(&mut self, _interval: f64) {
        //! set interval(second) at header

        self.header.interval = _interval;
    }

    pub fn set_logfile(&mut self, _logfile: String) {
        //! Set logfile path.
        self.logfile = _logfile;
    }

    pub fn draw_update(&mut self) {
        //! clear and update draw window.

        // draw
        clear();
        self.header.update();
        self.watch.draw_history();
        self.watch.update();
    }

    // @TODO: whileで回してる最中に受け付けた処理は廃棄するように
    pub fn get_event(&mut self) {
        //! start input reception event.

        mousemask(ALL_MOUSE_EVENTS as mmask_t, None);
        while !self.done {
            match self.rx.try_recv() {
                // get result, run self.update()
                Ok(Event::OutputUpdate(_cmd)) => self.update(_cmd),

                // get exit event
                Ok(Event::Exit) => self.done = true,

                // get signal
                Ok(Event::Signal(i)) => match i {
                    0x02 => self.exit(),
                    _ => {}
                },
                Ok(Event::Input(i)) => self.input_action(i),
                _ => {}
            };
            thread::sleep(Duration::from_millis(5));
        }
    }

    // TODO(blacknon): CURSOR_NORMAL_WINDOW時のみ受け付けるinput actionの作成
    fn input_action_normal_window(&mut self, _input: i32) {
        //! Input action at watch/history window.

        match _input {
            // Mouse
            KEY_MOUSE => {
                let mut mevent = MEVENT {
                    id: 0,
                    x: 0,
                    y: 0,
                    z: 0,
                    bstate: 0,
                };
                let _error = getmouse(&mut mevent);
                if _error == 0 {
                    self.mouse_action(mevent)
                }
            }

            // Screen Resize
            KEY_RESIZE => self.watch.resize(),

            // change active pad
            0x09 => self.toggle_pad(), // Tab

            // pad up/down
            KEY_UP => self.up(),     // Arrow Up
            KEY_DOWN => self.down(), // Arrow Down

            // pad pageup/pagedown
            KEY_PPAGE => self.page_up(),   // Page up
            KEY_NPAGE => self.page_down(), // Page down

            // toggle color mode
            0x63 => {
                // c(0x63)
                self.toggle_color();
                self.draw_update();
            }

            // change diff mode
            0x64 => {
                // d(0x64)
                self.toggle_diff();
                self.draw_update();
            }
            0x30 => {
                // 0(0x30)
                self.switch_diff(0);
                self.draw_update();
            }
            0x31 => {
                // 1(0x31)
                self.switch_diff(1);
                self.draw_update();
            }
            0x32 => {
                // 2(0x32)
                self.switch_diff(2);
                self.draw_update();
            }

            // show help window
            0x68 => {
                // h(0x68)
                self.toggle_show_help();
            }

            // search mode
            // 0x2f => {
            // /(0x2f)
            // }

            // change output
            KEY_F1 => {
                // F1
                self.set_output_type(::IS_STDOUT);
                self.draw_update();
            }
            KEY_F2 => {
                // F2
                self.set_output_type(::IS_STDERR);
                self.draw_update();
            }
            KEY_F3 => {
                // F3
                self.set_output_type(::IS_OUTPUT);
                self.draw_update();
            }

            // exit this program
            0x71 => self.exit(), // q(0x71)

            _ => {}
        }
    }

    // TODO(blacknon): CURSOR_HELP_WINDOW時のみ受け付けるinput actionの作成
    fn input_action_help_window(&mut self, _input: i32) {
        //! Input action at help window.

        match _input {
            // show help window
            0x68 => {
                // h(0x68)
                self.toggle_show_help();
            }

            _ => {}
        }
    }

    // TODO(blacknon): CURSOR_INPUT_KEYWORD時のみ受け付けるinput actionの作成
    fn input_action(&mut self, _input: i32) {
        //! Input action signal.

        // TODO(blacknon): cursor_modeに応じて受け付けるkey inputの処理を切り替える
        match self.cursor_mode {
            ::CURSOR_NORMAL_WINDOW => self.input_action_normal_window(_input),
            ::CURSOR_HELP_WINDOW => self.input_action_help_window(_input),
            // ::CURSOR_INPUT_KEYWORD => {},
            _ => {}
        }
    }

    fn mouse_action(&mut self, _mevent: MEVENT) {
        //! mouse action signal.

        let _mouse_event = _mevent.bstate as i32;
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.screen, &mut max_y, &mut max_x);

        // mouse is not on header
        if _mevent.y >= 2 {
            match _mouse_event {
                // mouse left button click
                BUTTON1_CLICKED => {
                    // MEMO: マウスカーソルがhistory領域にいる場合
                    if max_x - ::HISTORY_WIDTH < _mevent.x {
                        // MEMO: マウスが選択した行を記録
                        let _mouse_select_line = _mevent.y - 2 + self.watch.history_pad_position;

                        if self.watch.count >= _mouse_select_line {
                            self.watch.selected = _mouse_select_line;

                            // update draw
                            self.watch.draw_history();
                            self.watch.update();
                        }
                    }
                }

                // mouse wheel up
                BUTTON4_PRESSED => {
                    if max_x - ::HISTORY_WIDTH < _mevent.x {
                        self.header.active_pad = ::IS_HISTORY_PAD;
                    } else {
                        self.header.active_pad = ::IS_WATCH_PAD;
                    }
                    self.header.update();
                    self.up();
                }

                // mouse wheel down
                BUTTON5_PRESSED => {
                    if max_x - ::HISTORY_WIDTH < _mevent.x {
                        self.header.active_pad = ::IS_HISTORY_PAD;
                    } else {
                        self.header.active_pad = ::IS_WATCH_PAD;
                    }
                    self.header.update();
                    self.down();
                }
                _ => {}
            }
        }
    }
}

fn get_lang() -> String {
    //! get language.

    let key = "LANG";
    match env::var(key) {
        Ok(val) => return val,
        _ => return String::new(),
    }
}
