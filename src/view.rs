// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use crossbeam_channel::{Receiver, Sender};
use std::time::{Duration, Instant};
use crossterm::{
    event::{Event, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    sync::{Arc, RwLock},
};
use tui::{backend::CrosstermBackend, Terminal};
use std::os::unix::io::AsRawFd; // 追加

// local module
use crate::app::App;
use crate::common::{DiffMode, OutputMode};
use crate::event::AppEvent;
use crate::keymap::{Keymap, default_keymap};
use termios::*;

// local const
use crate::Interval;
use crate::DEFAULT_TAB_SIZE;

/// Struct at run hwatch on tui
#[derive(Clone)]
pub struct View {
    after_command: String,
    interval: Interval,
    tab_size: u16,
    limit: u32,
    keymap: Keymap,
    beep: bool,
    border: bool,
    scroll_bar: bool,
    mouse_events: bool,
    color: bool,
    show_ui: bool,
    show_help_banner: bool,
    line_number: bool,
    reverse: bool,
    output_mode: OutputMode,
    diff_mode: DiffMode,
    is_only_diffline: bool,
    log_path: String,
}

///
impl View {
    pub fn new(interval: Interval) -> Self {
        Self {
            after_command: "".to_string(),
            interval,
            tab_size: DEFAULT_TAB_SIZE,
            limit: 0,
            keymap: default_keymap(),
            beep: false,
            border: false,
            scroll_bar: false,
            mouse_events: false,
            color: false,
            show_ui: true,
            show_help_banner: true,
            line_number: false,
            reverse: false,
            output_mode: OutputMode::Output,
            diff_mode: DiffMode::Disable,
            is_only_diffline: false,
            log_path: "".to_string(),
        }
    }

    pub fn set_after_command(mut self, command: String) -> Self {
        self.after_command = command;
        self
    }

    pub fn set_interval(mut self, interval: Arc<RwLock<f64>>) -> Self {
        self.interval = interval;
        self
    }

    pub fn set_tab_size(mut self, tab_size: u16) -> Self {
        self.tab_size = tab_size;
        self
    }

    pub fn set_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    pub fn set_keymap(mut self, keymap: Keymap) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn set_beep(mut self, beep: bool) -> Self {
        self.beep = beep;
        self
    }

    pub fn set_border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn set_scroll_bar(mut self, scroll_bar: bool) -> Self {
        self.scroll_bar = scroll_bar;
        self
    }

    pub fn set_mouse_events(mut self, mouse_events: bool) -> Self {
        self.mouse_events = mouse_events;
        self
    }

    pub fn set_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    pub fn set_show_ui(mut self, show_ui: bool) -> Self {
        self.show_ui = show_ui;
        self
    }

    pub fn set_show_help_banner(mut self, show_help_banner: bool) -> Self {
        self.show_help_banner = show_help_banner;
        self
    }

    pub fn set_line_number(mut self, line_number: bool) -> Self {
        self.line_number = line_number;
        self
    }

    pub fn set_reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    pub fn set_output_mode(mut self, output_mode: OutputMode) -> Self {
        self.output_mode = output_mode;
        self
    }

    pub fn set_diff_mode(mut self, diff_mode: DiffMode) -> Self {
        self.diff_mode = diff_mode;
        self
    }

    pub fn set_only_diffline(mut self, only_diffline: bool) -> Self {
        self.is_only_diffline = only_diffline;
        self
    }

    pub fn set_logfile(mut self, log_path: String) -> Self {
        self.log_path = log_path;
        self
    }

    pub fn start(
        &mut self,
        tx: Sender<AppEvent>,
        rx: Receiver<AppEvent>,
    ) -> Result<(), Box<dyn Error>> {
        // Setup Terminal
        enable_raw_mode()?;
        // set_noncanonical_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let _ = terminal.clear();

        {
            let input_tx = tx.clone();
            let mut last_event_time = Instant::now();
            let _ = std::thread::spawn(move || loop {
                match send_input(input_tx.clone(), &last_event_time) {
                    Ok(true) => last_event_time = Instant::now(),
                    Ok(false) => {}
                    Err(_) => {},
                }
            });
        }

        // Create App
        let mut app = App::new(tx, rx, self.interval.clone());

        // set keymap
        app.set_keymap(self.keymap.clone());

        // set after command
        app.set_after_command(self.after_command.clone());

        // set mouse events
        // Windows mouse capture implemention requires EnableMouseCapture be invoked before DisableMouseCatpure can be used
        // https://github.com/crossterm-rs/crossterm/issues/660
        if cfg!(target_os = "windows") {
            execute!(terminal.backend_mut(), EnableMouseCapture)?;
        }
        app.set_mouse_events(self.mouse_events);

        // set limit
        app.set_limit(self.limit);

        // set beep
        app.set_beep(self.beep);

        // set border
        app.set_border(self.border);
        app.set_scroll_bar(self.scroll_bar);

        // set logfile path.
        app.set_logpath(self.log_path.clone());

        // set color
        app.set_ansi_color(self.color);

        app.show_history(self.show_ui);
        app.show_ui(self.show_ui);
        app.show_help_banner(self.show_help_banner);

        app.set_tab_size(self.tab_size);

        // set line_number
        app.set_line_number(self.line_number);

        // set reverse
        app.set_reverse(self.reverse);

        // set output mode
        app.set_output_mode(self.output_mode);

        // set diff mode
        app.set_diff_mode(self.diff_mode);
        app.set_is_only_diffline(self.is_only_diffline);

        // Run App
        let res = app.run(&mut terminal);

        // exit app and restore terminal
        // reset_terminal_mode()?;
        restore_terminal();
        if let Err(err) = res {
            println!("{err:?}")
        }

        Ok(())
    }
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        _ => return,
    };

    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();
}

fn send_input(tx: Sender<AppEvent>, last_event_time: &Instant) -> io::Result<bool> {
    if crossterm::event::poll(Duration::from_millis(100))? {
        let now = Instant::now();

        // read event
        let event = crossterm::event::read()?;

        let mut is_mouse_event = false;
        if let Event::Mouse(_) = event {
            is_mouse_event = true;
        }

        // check if 100ms has passed since the last event
        if now.duration_since(*last_event_time) >= Duration::from_millis(5) {
            match event {
                Event::Key(key) => {
                    if key.code != crossterm::event::KeyCode::Esc || now.duration_since(*last_event_time) >= Duration::from_millis(500) {
                        let _ = tx.send(AppEvent::TerminalEvent(event));
                    }
                },

                _ => {
                    let _ = tx.send(AppEvent::TerminalEvent(event));
                },
            }

            // buffer clearing
            while crossterm::event::poll(Duration::from_millis(0))? {
                let _ = crossterm::event::read()?;
            }

            return Ok(is_mouse_event);
        } else {
            // buffer clearing
            while crossterm::event::poll(Duration::from_millis(0))? {
                let _ = crossterm::event::read()?;
            }

            return Ok(is_mouse_event);
        }
    }
    Ok(false)
}

// fn set_noncanonical_mode() -> io::Result<()> {
//     let stdin_fd = io::stdin().as_raw_fd();
//     let mut termios = Termios::from_fd(stdin_fd)?;

//     // 非カノニカルモードを設定（ICANONを無効化）
//     termios.c_lflag &= !(ICANON | ECHO); // ICANONとECHOを無効化して即時入力を有効にする
//     termios.c_cc[VMIN] = 1; // 最低1文字の入力で処理する
//     termios.c_cc[VTIME] = 0; // タイムアウトはなし

//     // 設定を適用
//     tcsetattr(stdin_fd, TCSANOW, &termios)?;
//     Ok(())
// }

// fn reset_terminal_mode() -> io::Result<()> {
//     let stdin_fd = io::stdin().as_raw_fd();
//     let mut termios = Termios::from_fd(stdin_fd)?;

//     // ターミナルの設定をリセット
//     termios.c_lflag |= ICANON | ECHO; // カノニカルモードとエコーを再有効化
//     tcsetattr(stdin_fd, TCSANOW, &termios)?;
//     Ok(())
// }
