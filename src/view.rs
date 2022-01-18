// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[warn(unused_doc_comments)]
// module
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

// local module
use event::AppEvent;
use exec::CommandResult;
use header::HeaderArea;
use history::HistoryArea;
use watch::WatchArea;

///
#[derive(Clone, Copy)]
pub enum ActiveArea {
    Watch,
    History,
}

///
#[derive(Clone, Copy)]
pub enum ActiveWindow {
    Normal,
    Help,
}

///
#[derive(Clone, Copy)]
pub enum DiffMode {
    Disable,
    Watch,
    Line,
}

///
#[derive(Clone, Copy)]
pub enum OutputMode {
    Output,
    Stdout,
    Stderr,
}

///
#[derive(Clone, Copy)]
enum InputMode {
    None,
    Filter,
    Search,
}

/// Struct at watch view window.
pub struct App<'a> {
    ///
    timeout: std::time::Duration,

    ///
    area: ActiveArea,

    ///
    window: ActiveWindow,

    ///
    ansi_color: bool,

    ///
    input_mode: InputMode,

    ///
    output_mode: OutputMode,

    ///
    results: Mutex<Vec<CommandResult>>,

    ///
    header_area: HeaderArea<'a>,

    ///
    history_area: HistoryArea,

    ///
    watch_area: WatchArea<'a>,

    /// It is a flag value to confirm the done of the app.
    /// If `true`, exit app.
    pub done: bool,

    /// logfile path.
    pub logfile: String,

    pub tx: Sender<AppEvent>,
    pub rx: Receiver<AppEvent>,
}

/// Trail at watch view window.
impl<'a> App<'a> {
    pub fn new(tx: Sender<AppEvent>, rx: Receiver<AppEvent>) -> Self {
        ///! method at create new view trail.
        Self {
            timeout: Duration::from_millis(10),
            area: ActiveArea::History,
            window: ActiveWindow::Normal,

            ansi_color: false,
            input_mode: InputMode::None,
            output_mode: OutputMode::Output,

            results: Mutex::new(vec![]),

            header_area: HeaderArea::new(),
            history_area: HistoryArea::new(),
            watch_area: WatchArea::new(),
            done: false,
            logfile: "".to_string(),
            tx: tx,
            rx: rx,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        self.history_area.next();

        loop {
            if self.done {
                return Ok(());
            }

            // Draw data
            terminal.draw(|f| self.draw(f))?;

            // get event
            match self.rx.try_recv() {
                // Get terminal event.
                Ok(AppEvent::TerminalEvent(terminal_event)) => self.get_event(terminal_event),

                // Get command result.
                Ok(AppEvent::OutputUpdate(exec_result)) => self.update_result(exec_result),

                // get exit event
                Ok(AppEvent::Exit) => self.done = true,

                _ => {}
            }
        }
    }

    fn get_area<B: Backend>(&mut self, f: &mut Frame<B>) {
        // get Area's chunks
        let top_chunks = Layout::default()
            .constraints([Constraint::Length(2), Constraint::Max(0)].as_ref())
            .split(f.size());

        let main_chanks = Layout::default()
            .constraints(
                [
                    Constraint::Max(f.size().width - ::HISTORY_WIDTH),
                    Constraint::Length(::HISTORY_WIDTH),
                ]
                .as_ref(),
            )
            .direction(Direction::Horizontal)
            .split(top_chunks[1]);

        let areas = [top_chunks[0], main_chanks[0], main_chanks[1]];

        self.header_area.set_area(areas[0]);
        self.watch_area.set_area(areas[1]);
        self.history_area.set_area(areas[2]);
    }

    fn get_event(&mut self, terminal_event: crossterm::event::Event) {
        match self.input_mode {
            InputMode::None => self.get_input_key(terminal_event),
            InputMode::Filter => {}
            InputMode::Search => {}
        }
    }

    fn set_output_data(&mut self, num: usize) {
        let results = self.results.lock().unwrap();
        let count = results.len();

        let mut target = num;
        if num >= 1 {
            target = target - 1;
        }

        // outpu text
        let output_data: &str;
        match self.output_mode {
            OutputMode::Output => output_data = &results[target].output,
            OutputMode::Stdout => output_data = &results[target].stdout,
            OutputMode::Stderr => output_data = &results[target].stderr,
        }

        if count > target {
            self.watch_area.update_output(output_data);
        }
    }

    fn set_output_mode(&mut self, mode: OutputMode) {
        self.output_mode = mode;
        self.header_area.set_output_mode(mode);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.get_area(f);

        // Draw header area.
        self.header_area.draw(f);

        // Draw watch area.
        self.watch_area.draw(f);

        // Draw history area
        self.history_area.draw(f);
    }

    pub fn update_result(&mut self, _result: CommandResult) {
        // diff output data.

        // append results
        let mut results = self.results.lock().unwrap();
        results.insert(0, _result.clone());

        // update HistoryArea
        let _timestamp = &results[0].timestamp;
        self.history_area.update(_timestamp.to_string());

        // update selected
        let selected = self.history_area.get_state_select();
        if selected != 0 {
            self.history_area.previous();
        }

        // update HeaderArea
        self.header_area.set_current_result(_result.clone());
        self.header_area.update();

        // update WatchArea
        drop(results);
        self.set_output_data(selected);

        //
        // if selected == 0 {
        //     self.watch_area.update(&_result.output);
        // }
    }

    fn get_input_key(&mut self, terminal_event: crossterm::event::Event) {
        match self.window {
            ActiveWindow::Normal => {
                match terminal_event {
                    // up
                    Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    }) => self.input_key_up(),

                    // down
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    }) => self.input_key_down(),

                    // pgup

                    // pgdn

                    // left

                    // right

                    // c

                    // d

                    // o
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('o'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_output(),

                    // Tab ... Toggle Area(Watch or History).
                    Event::Key(KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_area(),

                    // Common input key
                    // h ... toggel help window.

                    // q ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                    }) => self
                        .tx
                        .send(AppEvent::Exit)
                        .expect("send error hwatch exit."),

                    // Ctrl + C ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => self
                        .tx
                        .send(AppEvent::Exit)
                        .expect("send error hwatch exit."),

                    _ => {}
                }
            }
            ActiveWindow::Help => {
                match terminal_event {
                    // Common input key
                    // h ... toggel help window.

                    // q ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                    }) => self
                        .tx
                        .send(AppEvent::Exit)
                        .expect("send error hwatch exit."),

                    // Ctrl + C ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => self
                        .tx
                        .send(AppEvent::Exit)
                        .expect("send error hwatch exit."),

                    _ => {}
                }
            }
        }
    }

    fn toggle_area(&mut self) {
        match self.window {
            ActiveWindow::Normal => {
                match self.area {
                    ActiveArea::Watch => self.area = ActiveArea::History,
                    ActiveArea::History => self.area = ActiveArea::Watch,
                }

                // set active window to header.
                self.header_area.set_active_area(self.area.clone());
                self.header_area.update();
            }
            _ => {}
        }
    }

    fn toggle_output(&mut self) {
        match self.output_mode {
            OutputMode::Output => self.set_output_mode(OutputMode::Stdout),
            OutputMode::Stdout => self.set_output_mode(OutputMode::Stderr),
            OutputMode::Stderr => self.set_output_mode(OutputMode::Output),
        }
    }

    fn input_key_up(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {}
                ActiveArea::History => {
                    // move next history
                    self.history_area.next();

                    // get now selected history
                    let selected = self.history_area.get_state_select();
                    self.set_output_data(selected);
                }
            },
            ActiveWindow::Help => {}
        }
    }

    fn input_key_down(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {}
                ActiveArea::History => {
                    // move previous history
                    self.history_area.previous();

                    // get now selected history
                    let selected = self.history_area.get_state_select();
                    self.set_output_data(selected);
                }
            },
            ActiveWindow::Help => {}
        }
    }

    fn input_key_left(&mut self) {}

    fn input_key_right(&mut self) {}
}

/// start hwatch app view.
pub fn start(tx: Sender<AppEvent>, rx: Receiver<AppEvent>) -> Result<(), Box<dyn Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    {
        let input_tx = tx.clone();
        let _ = std::thread::spawn(move || loop {
            let _ = send_input(input_tx.clone());
        });
    }

    // Create App
    let mut app = App::new(tx, rx);

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

fn send_input(tx: Sender<AppEvent>) -> io::Result<()> {
    let timeout = Duration::from_millis(5);
    if crossterm::event::poll(timeout)? {
        let event = crossterm::event::read().expect("failed to read crossterm event");
        let _ = tx.clone().send(AppEvent::TerminalEvent(event));
    }

    Ok(())
}
