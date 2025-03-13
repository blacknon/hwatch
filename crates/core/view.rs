// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// module
use crossbeam_channel::{Receiver, Sender};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::Duration;
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

// non blocking io
#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
use nix::fcntl::{fcntl, FcntlArg::*, OFlag};
#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
use std::{io::stdin, os::unix::io::AsRawFd};

// local module
use crate::app::App;
use crate::common::{DiffMode, OutputMode};
use crate::event::AppEvent;
use crate::exec::CommandResult;
use crate::keymap::{default_keymap, Keymap};

// local const
use crate::SharedInterval;
use crate::DEFAULT_TAB_SIZE;

/// Struct at run hwatch on tui
#[derive(Clone)]
pub struct View {
    after_command: String,
    interval: SharedInterval,
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
    enable_summary_char: bool,
    log_path: String,
}

///
impl View {
    pub fn new(interval: SharedInterval) -> Self {
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
            enable_summary_char: false,
            log_path: "".to_string(),
        }
    }

    pub fn set_after_command(mut self, command: String) -> Self {
        self.after_command = command;
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

    pub fn set_enable_summary_char(mut self, enable_summary_char: bool) -> Self {
        self.enable_summary_char = enable_summary_char;
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
        exist_results: Vec<CommandResult>,
    ) -> Result<(), Box<dyn Error>> {
        // Setup Terminal
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(backend)?;
        let _ = terminal.clear();

        {
            let input_tx = tx.clone();
            let _ = std::thread::spawn(move || {
                // non blocking io
                #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
                loop {
                    let _ = send_input(input_tx.clone());
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
        // Windows mouse capture implemention requires EnableMouseCapture be invoked before DisableMouseCapture can be used
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

        // set enable summary char
        app.set_enable_summary_char(self.enable_summary_char);

        // set exist results
        app.add_results(exist_results);

        // Run App
        let res = app.run(&mut terminal);

        // exit app and restore terminal
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

// TODO(blacknon): `Os { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" }`で終了してしまう場合があるので、どこで終了されてしまっているかを調査、対処する
fn send_input(tx: Sender<AppEvent>) -> io::Result<()> {
    if crossterm::event::poll(Duration::from_millis(100))? {
        // non blocking mode
        #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
        set_nonblocking(true)?;

        // get event
        let result = crossterm::event::read();

        // blocking mode
        #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
        set_nonblocking(false)?;

        if let Ok(event) = result {
            let _ = tx.send(AppEvent::TerminalEvent(event));
        }
    }
    Ok(())
}

#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
pub fn set_nonblocking(is_nonblocking: bool) -> nix::Result<()> {
    let stdin_fd = stdin().as_raw_fd();
    let flags = fcntl(stdin_fd, F_GETFL)?;

    if is_nonblocking {
        let new_flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
        fcntl(stdin_fd, F_SETFL(new_flags))?;
    } else {
        let new_flags = OFlag::from_bits_truncate(flags) & !OFlag::O_NONBLOCK;
        fcntl(stdin_fd, F_SETFL(new_flags))?;
    }

    Ok(())
}
