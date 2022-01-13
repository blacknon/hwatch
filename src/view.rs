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
    widgets::Block,
    Frame, Terminal,
};

// local module
use exec;
use exec::ExecEvent;
use header::HeaderArea;
use history::HistoryArea;
use watch::WatchArea;

///
pub enum ActiveArea {
    Watch,
    History,
}

///
pub enum ActiveWindow {
    Normal,
    Help,
}

///
pub enum DiffMode {
    Disable,
    Watch,
    Line,
}

///
pub enum OutputMode {
    Output,
    Stdout,
    Stderr,
}

///
enum InputMode {
    None,
    Filter,
    Search,
}

/// Struct at watch view window.
pub struct App<'a> {
    // debug. after delete
    pub area_size: [tui::layout::Rect; 3],

    ///
    area: ActiveArea,

    ///
    window: ActiveWindow,

    ///
    input: InputMode,

    ///
    ansi_color: bool,

    ///
    results: Mutex<Vec<exec::Result>>,

    ///
    history: Vec<String>,

    ///
    current: i32,

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

    pub tx: Sender<ExecEvent>,
    pub rx: Receiver<ExecEvent>,
}

/// Trail at watch view window.
impl<'a> App<'a> {
    pub fn new(tx: Sender<ExecEvent>, rx: Receiver<ExecEvent>) -> Self {
        ///! method at create new view trail.
        Self {
            area_size: [
                tui::layout::Rect::new(0, 0, 0, 0),
                tui::layout::Rect::new(0, 0, 0, 0),
                tui::layout::Rect::new(0, 0, 0, 0),
            ],
            area: ActiveArea::History,
            window: ActiveWindow::Normal,
            input: InputMode::None,
            ansi_color: false,
            results: Mutex::new(vec![]),
            history: vec![],
            current: 0,
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
        loop {
            if self.done {
                return Ok(());
            }

            // Draw data
            terminal.draw(|f| self.draw(f))?;

            // get input
            let timeout = Duration::from_millis(5);
            if crossterm::event::poll(timeout)? {
                match self.input {
                    InputMode::None => match self.window {
                        ActiveWindow::Help => {}
                        ActiveWindow::Normal => {}
                    },

                    InputMode::Filter => {}
                    InputMode::Search => {}
                }

                match event::read().unwrap() {
                    // Common input key
                    // Input Ctrl + C
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => return Ok(()),

                    // Input Tab
                    Event::Key(KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                    }) => return Ok(()),

                    // Input ESC
                    _ => {}
                }
            }

            // get result data
            match self.rx.try_recv() {
                // Get command result.
                Ok(ExecEvent::OutputUpdate(exec_result)) => self.update_result(exec_result),

                // get exit event
                Ok(ExecEvent::Exit) => self.done = true,

                _ => {}
            }

            // let aaa = event::read().unwrap();

            // match self.active {
            //     ActiveArea::Watch => match key.code {
            //         KeyCode::Char('e') => {
            //             self.input_mode = InputMode::Editing;
            //         }
            //         KeyCode::Char('q') => {
            //             return Ok(());
            //         }
            //         _ => {}
            //     },
            //     ActiveArea::History => match key.code {
            //         KeyCode::Enter => {
            //             self.messages.push(self.input.drain(..).collect());
            //         }
            //         KeyCode::Char(c) => {
            //             self.input.push(c);
            //         }
            //         KeyCode::Backspace => {
            //             self.input.pop();
            //         }
            //         KeyCode::Esc => {
            //             self.input_mode = InputMode::Normal;
            //         }
            //         _ => {}
            //     },
            // }

            // Ok(())
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
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.get_area(f);

        // Draw header area.
        self.header_area.draw(f);

        // Draw watch area.
        self.watch_area.draw(f);

        let block = Block::default().title("history");
        f.render_widget(block, self.area_size[2]);
    }

    pub fn update_result(&mut self, _result: exec::Result) {
        // diff output data.

        // append results
        let mut results = self.results.lock().unwrap();
        results.insert(0, _result.clone());
        let count_results = results.len() as i32;

        // append history
        self.history.push(_result.timestamp.to_string());

        // update current
        self.current += 1;

        // update HeaderArea
        self.header_area.update(_result.clone(), &self.area);

        // update HistoryArea
        self.history_area.update(&self.history, self.current);

        // update WatchArea
        if self.current == count_results {
            self.watch_area.update(&_result.output);
        }
    }

    pub fn input_key_up(&mut self) {}

    pub fn input_Key_down(&mut self) {}

    pub fn input_key_left(&mut self) {}

    pub fn input_key_right(&mut self) {}
}

/// start hwatch app view.
pub fn start(tx: Sender<ExecEvent>, rx: Receiver<ExecEvent>) -> Result<(), Box<dyn Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
