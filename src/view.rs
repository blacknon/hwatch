// Copyright (c) 2021 Blacknon. All rights reserved.
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
    color: bool,
    line_number: bool,
    watch_diff: bool,
    log_path: String,
}

///
impl View {
    pub fn new() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            color: false,
            line_number: false,
            watch_diff: false,
            log_path: "".to_string(),
        }
    }

    pub fn set_interval(mut self, interval: f64) -> Self {
        self.interval = interval;
        self
    }

    pub fn set_color(mut self, color: bool) -> Self {
        self.color = color;
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
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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

        // set logfile path.
        app.set_logpath(self.log_path.clone());

        // set color
        app.set_ansi_color(self.color);

        // set line_number
        app.set_line_number(self.line_number);

        // set watch diff
        if self.watch_diff {
            app.set_diff_mode(DiffMode::Watch);
        }

        // Run App
        let res = app.run(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }
}

fn send_input(tx: Sender<AppEvent>) -> io::Result<()> {
    let event = crossterm::event::read().expect("failed to read crossterm event");
    let _ = tx.send(AppEvent::TerminalEvent(event));
    Ok(())
}
