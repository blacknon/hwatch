// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crossbeam_channel::{Receiver, Sender};
// module
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::symbols::border;
use std::{
    error::Error,
    io,
    sync::{Arc, RwLock},
};
use tui::{backend::CrosstermBackend, Terminal};

// local module
use crate::app::App;
use crate::common::{DiffMode, OutputMode};
use crate::event::AppEvent;

// local const
use crate::Interval;
use crate::DEFAULT_TAB_SIZE;

/// Struct at run hwatch on tui
#[derive(Clone)]
pub struct View {
    after_command: String,
    interval: Interval,
    tab_size: u16,
    beep: bool,
    border: bool,
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
            beep: false,
            border: false,
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

    pub fn set_beep(mut self, beep: bool) -> Self {
        self.beep = beep;
        self
    }

    pub fn set_border(mut self, border: bool) -> Self {
        self.border = border;
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
        let mut app = App::new(tx, rx, self.interval.clone(), self.mouse_events);

        // set after command
        app.set_after_command(self.after_command.clone());

        // set beep
        app.set_beep(self.beep);

        // set border
        app.set_border(self.border);

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
