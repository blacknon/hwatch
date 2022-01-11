// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[warn(unused_doc_comments)]
// module
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
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
    thread,
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame, Terminal,
};

// local module
use exec;
use signal::AppEvent;
use watch::WatchArea;

/// Struct at watch view window.
pub struct App<'a> {
    pub area_size: [tui::layout::Rect; 3],

    ///
    ///
    results: Mutex<Vec<exec::Result>>,

    ///
    pub watch_area: WatchArea<'a>,

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
            area_size: [
                tui::layout::Rect::new(0, 0, 0, 0),
                tui::layout::Rect::new(0, 0, 0, 0),
                tui::layout::Rect::new(0, 0, 0, 0),
            ],
            watch_area: WatchArea::new(),
            results: Mutex::new(vec![]),
            done: false,
            logfile: "".to_string(),
            tx: tx,
            rx: rx,
        }
    }

    pub fn get_area<B: Backend>(&mut self, f: &mut Frame<B>) {
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

        self.watch_area.set_area(areas[1]);
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let block = Block::default().title("header");
        f.render_widget(block, self.area_size[0]);

        // Draw watch area.
        self.watch_area.draw(f);

        let block = Block::default().title("history");
        f.render_widget(block, self.area_size[2]);
    }

    pub fn result_update(&mut self, result: exec::Result) {
        self.watch_area.update_data(&result.output);
    }
}

/// start hwatch app view.
pub fn start(tx: Sender<AppEvent>, rx: Receiver<AppEvent>) -> Result<(), Box<dyn Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App
    let mut app = App::new(tx, rx);

    // Run App
    run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App) {
    // get Area Size from terminal.Frame
    let mut frame = terminal.get_frame();
    app.get_area(&mut frame);

    while !app.done {
        // draw
        terminal.draw(|f| draw(f, &mut app));
        thread::sleep(Duration::from_millis(5));

        match app.rx.try_recv() {
            // get result, run self.update()
            Ok(AppEvent::OutputUpdate(exec_result)) => app.result_update(exec_result),

            // get exit event
            Ok(AppEvent::Exit) => app.done = true,

            // get signal
            Ok(AppEvent::Signal(i)) => match i {
                0x02 => app.done = true,
                _ => {}
            },
            // Ok(AppEvent::Input(i)) => app.input(i),
            Ok(AppEvent::Input(i)) => {}
            _ => {}
        };
        thread::sleep(Duration::from_millis(5));
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    app.draw(f)
}
