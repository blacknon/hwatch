// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crossbeam_channel::{Receiver, Sender};
// module
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

// local module
use crate::app::{App, DiffMode};
use crate::event::AppEvent;

// local const
use crate::DEFAULT_INTERVAL;

/// Struct at run hwatch on tui
#[derive(Clone)]
pub struct View {
    interval: f64,
    beep: bool,
    mouse_events: bool,
    color: bool,
    show_ui: bool,
    show_help_banner: bool,
    line_number: bool,
    watch_diff: bool,
    log_path: String,
}

///
impl View {
    pub fn new() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            beep: false,
            mouse_events: false,
            color: false,
            show_ui: true,
            show_help_banner: true,
            line_number: false,
            watch_diff: false,
            log_path: "".to_string(),
        }
    }

    pub fn set_interval(mut self, interval: f64) -> Self {
        self.interval = interval;
        self
    }

    pub fn set_beep(mut self, beep: bool) -> Self {
        self.beep = beep;
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

    pub fn set_watch_diff(mut self, watch_diff: bool) -> Self {
        self.watch_diff = watch_diff;
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
        ctrlc::set_handler(|| {
            // Runs on SIGINT, SIGTERM (kill), SIGHUP
            restore_terminal();
            // Exit code for SIGTERM (signal 15), not quite right if another signal is the cause.
            std::process::exit(128 + 15)
        })?;
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        if self.mouse_events {
            execute!(stdout, EnableMouseCapture)?;
        }
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let _ = terminal.clear();

        {
            let input_tx = tx.clone();
            let _ = std::thread::spawn(move || loop {
                let _ = send_input(input_tx.clone());
            });
        }

        // Create App
        let mut app = App::new(tx, rx);

        // set interval
        app.set_interval(self.interval);

        // set beep
        app.set_beep(self.beep);

        // set logfile path.
        app.set_logpath(self.log_path.clone());

        // set color
        app.set_ansi_color(self.color);

        app.show_history(self.show_ui);
        app.show_ui(self.show_ui);
        app.show_help_banner(self.show_help_banner);

        // set line_number
        app.set_line_number(self.line_number);

        // set watch diff
        if self.watch_diff {
            app.set_diff_mode(DiffMode::Watch);
        }

        // Run App
        let res = app.run(&mut terminal);
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

fn send_input(tx: Sender<AppEvent>) -> io::Result<()> {
    let event = crossterm::event::read().expect("failed to read crossterm event");
    let _ = tx.send(AppEvent::TerminalEvent(event));
    Ok(())
}
