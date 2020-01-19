// Copyright (c) 2019 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO(blacknon): キーワード検索機能の追加(v0.1.4)
//     - `/`でキーボード入力モードに
//     - `f`でフィルターモードの切り替えか？(ハイライトモードとフィルタリングモード)
//     - ESCで元に戻す
//     - lessみたいに、キーワードの検索が行えるようにする
//     - 検索方式として、`ハイライト方式`及び`絞り込み方式`の2つが必要？

// TODO(blacknon): ヘルプ機能の追加(v0.1.4)
//     - `h`でヘルプウィンドウの表示をする
//     - `h`キーを再度入力することで切り替える
//     - `h`キーでヘルプウィンドウを表示するという注意書きを一番下の行に表示させる

// TODO(blacknon): キー入力の変更機能を追加？(v0.1.5？)
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

impl View {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
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

    //
    fn exit(&mut self) {
        self.watch.exit();
        let _ = self.tx.send(Event::Exit);
    }

    //
    fn update(&mut self, _result: Result) {
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
    }

    // toggle ansi color mode
    fn toggle_color(&mut self) {
        if self.watch.color {
            self.watch.color = false
        } else {
            self.watch.color = true
        }

        // update header status
        self.header.color = self.watch.color;
    }

    // toggle show help window
    fn toggle_show_help(&mut self) {
        // check self.cursor_mode
        if self.cursor_mode == ::CURSOR_NORMAL_WINDOW {
            self.cursor_mode = ::CURSOR_HELP_WINDOW;
            // TODO(blacknon): help windowの表示処理
            self.watch.show_help_window();

        } else if self.cursor_mode == ::CURSOR_HELP_WINDOW {
            self.cursor_mode = ::CURSOR_NORMAL_WINDOW;
            // TODO(blacknon): help windowを非表示にする処理
            self.watch.show_help_window();

        }
        // TODO(blacknon): 共通系(ウィンドウ等の更新系処理)

        self.watch.update();
    }

    // toggle diff mode
    fn toggle_diff(&mut self) {
        // add num
        let mut now_diff = self.watch.diff;
        now_diff += 1;

        self.switch_diff(now_diff % 3);
    }

    // switch diff mode.
    pub fn switch_diff(&mut self, _diff: i32) {
        // set value
        self.watch.diff = _diff;
        self.header.diff = self.watch.diff;
    }

    // change active pad (history/watch)
    fn toggle_pad(&mut self) {
        // add num
        let mut now_pad = self.header.active_pad;
        now_pad += 3;

        self.header.active_pad = now_pad % 2;
        self.header.update();
    }

    // up action
    fn up(&mut self) {
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

    // down action
    fn down(&mut self) {
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

    // set color at header and watch pad
    fn set_output_type(&mut self, _output_type: i32) {
        // set value
        self.watch.output_type = _output_type;
        self.header.output = _output_type;
    }

    // set color at header and watch pad
    pub fn set_color(&mut self, _color: bool) {
        // set value
        self.watch.color = _color;
        self.header.color = _color;
    }

    // set command at header and watch pad
    pub fn set_command(&mut self, _command: String) {
        self.header.command = _command;
    }

    // set interval at header
    pub fn set_interval(&mut self, _interval: u64) {
        self.header.interval = _interval;
    }

    // set logfile
    pub fn set_logfile(&mut self, _logfile: String) {
        self.logfile = _logfile;
    }

    // clear and update draw
    pub fn draw_update(&mut self) {
        // draw
        clear();
        self.header.update();
        self.watch.draw_history();
        self.watch.update();
    }

    // start input reception
    // @TODO: whileで回してる最中に受け付けた処理は廃棄するように
    pub fn get_event(&mut self) {
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

    //
    fn input_action(&mut self, _input: i32) {
        // TODO(blacknon): cursor_modeに応じて受け付けるkey inputの処理を切り替える

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

    fn mouse_action(&mut self, _mevent: MEVENT) {
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
    let key = "LANG";
    match env::var(key) {
        Ok(val) => return val,
        _ => return String::new(),
    }
}
