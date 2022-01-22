// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[warn(unused_doc_comments)]
// module
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
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
    text::Spans,
    Frame, Terminal,
};

// local module
use common::differences_result;
use diff;
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
    Word,
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
    diff_mode: DiffMode,

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
    ///
    pub fn new(tx: Sender<AppEvent>, rx: Receiver<AppEvent>) -> Self {
        // method at create new view trail.
        Self {
            area: ActiveArea::History,
            window: ActiveWindow::Normal,

            ansi_color: false,
            input_mode: InputMode::None,
            output_mode: OutputMode::Output,
            diff_mode: DiffMode::Disable,

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
            match self.rx.recv_timeout(Duration::from_secs(60)) {
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

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.get_area(f);

        // Draw header area.
        self.header_area.draw(f);

        // Draw watch area.
        self.watch_area.draw(f);

        // Draw history area
        self.history_area.draw(f);
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

    /// Set the history to be output to WatchArea.
    fn set_output_data(&mut self, num: usize) {
        let results = self.results.lock().unwrap();
        let text: &str;
        let mut output_data = vec![];

        let mut target: usize = num;
        if num >= 1 {
            target = target - 1;
        }

        // check results over target...
        if results.len() <= target {
            return;
        }

        match self.output_mode {
            OutputMode::Output => text = &results[target].output,
            OutputMode::Stdout => text = &results[target].stdout,
            OutputMode::Stderr => text = &results[target].stderr,
        }

        let old_target = target + 1;
        let text_old: &str;
        if results.len() > old_target {
            match self.output_mode {
                OutputMode::Output => text_old = &results[old_target].output,
                OutputMode::Stdout => text_old = &results[old_target].stdout,
                OutputMode::Stderr => text_old = &results[old_target].stderr,
            }
        } else {
            text_old = "";
        }

        match self.diff_mode {
            DiffMode::Disable => {
                let lines = text.split("\n");
                for l in lines {
                    match self.ansi_color {
                        false => {
                            output_data.push(Spans::from(String::from(l)));
                        }

                        true => {
                            let data =
                                ansi4tui::bytes_to_text(format!("{}\n", l).as_bytes().to_vec());

                            for d in data.lines {
                                output_data.push(d);
                            }
                        }
                    }
                }
            }

            DiffMode::Watch => {
                output_data = diff::get_watch_diff(self.ansi_color, &text_old, &text);
            }

            DiffMode::Line => {
                output_data = diff::get_line_diff(self.ansi_color, &text_old, &text);
            }

            DiffMode::Word => {
                output_data = diff::get_word_diff(self.ansi_color, &text_old, &text);
            }
        }
        self.watch_area.update_output(output_data);
    }

    fn set_output_mode(&mut self, mode: OutputMode) {
        self.output_mode = mode;
        self.header_area.set_output_mode(mode);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    fn set_ansi_color(&mut self, ansi_color: bool) {
        self.ansi_color = ansi_color;

        // TODO: diffでcolorが使えるようになったら修正
        if self.ansi_color {
            self.diff_mode = DiffMode::Disable;
            self.set_diff_mode(self.diff_mode);
        }

        self.header_area.set_ansi_color(ansi_color);
        self.header_area.update();
        self.watch_area.set_ansi_color(ansi_color);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    fn set_interval(&mut self, interval: f64) {
        self.header_area.set_interval(interval);
    }

    fn set_diff_mode(&mut self, diff_mode: DiffMode) {
        self.diff_mode = diff_mode;

        // TODO: diffでcolorを使えるようになったら修正
        match self.diff_mode {
            DiffMode::Disable => {}
            _ => self.set_ansi_color(false),
        }

        self.header_area.set_diff_mode(diff_mode);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    fn update_result(&mut self, _result: CommandResult) {
        // unlock self.results
        let mut results = self.results.lock().unwrap();

        // check results size.
        let mut latest_result: &CommandResult;
        let tmp_result = CommandResult {
            timestamp: "".to_string(),
            command: "".to_string(),
            status: true,
            output: "".to_string(),
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        latest_result = &tmp_result;
        if results.len() > 0 {
            // diff output data.
            latest_result = &results[0];
        }

        // update HeaderArea
        self.header_area.set_current_result(_result.clone());
        self.header_area.update();

        // check result diff
        let check_result_diff = differences_result(&latest_result, &_result);
        if check_result_diff {
            return;
        }

        // append results
        results.insert(0, _result.clone());

        // update HistoryArea
        let _timestamp = &results[0].timestamp;
        let _status = &results[0].status;
        self.history_area
            .update(_timestamp.to_string(), _status.clone());

        // update selected
        let mut selected = self.history_area.get_state_select();
        if selected != 0 {
            self.history_area.previous();
        }
        selected = self.history_area.get_state_select();

        // drop mutex
        drop(results);

        // update WatchArea
        self.set_output_data(selected)
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
                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    }) => self.input_key_left(),

                    // right
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    }) => self.input_key_right(),

                    // c
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_ansi_color(),

                    // d
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_diff_mode(),

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

    fn toggle_ansi_color(&mut self) {
        match self.ansi_color {
            true => self.set_ansi_color(false),
            false => self.set_ansi_color(true),
        }
    }

    fn toggle_diff_mode(&mut self) {
        match self.diff_mode {
            DiffMode::Disable => self.set_diff_mode(DiffMode::Watch),
            DiffMode::Watch => self.set_diff_mode(DiffMode::Line),
            DiffMode::Line => self.set_diff_mode(DiffMode::Word),
            DiffMode::Word => self.set_diff_mode(DiffMode::Disable),
        }
    }

    fn input_key_up(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {
                    // scroll up watch
                    self.watch_area.scroll_up(1);
                }
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
                ActiveArea::Watch => {
                    // scroll up watch
                    self.watch_area.scroll_down(1);
                }
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

    fn input_key_left(&mut self) {
        match self.window {
            ActiveWindow::Normal => {
                self.area = ActiveArea::Watch;

                // set active window to header.
                self.header_area.set_active_area(self.area.clone());
                self.header_area.update();
            }
            _ => {}
        }
    }

    fn input_key_right(&mut self) {
        match self.window {
            ActiveWindow::Normal => {
                self.area = ActiveArea::History;

                // set active window to header.
                self.header_area.set_active_area(self.area.clone());
                self.header_area.update();
            }
            _ => {}
        }
    }
}

/// start hwatch app view.
pub struct View {
    interval: f64,
}

impl View {
    pub fn new() -> Self {
        Self {
            interval: ::DEFAULT_INTERVAL,
        }
    }

    pub fn set_interval(&mut self, interval: f64) {
        self.interval = interval;
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
    let timeout = Duration::from_millis(5);
    if crossterm::event::poll(timeout)? {
        let event = crossterm::event::read().expect("failed to read crossterm event");
        let _ = tx.clone().send(AppEvent::TerminalEvent(event));
    }
    Ok(())
}
