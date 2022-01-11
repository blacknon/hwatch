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
use signal::ExecEvent;
use watch::WatchArea;

/// Struct at watch view window.
pub struct App<'a> {
    /// frame area data.
    /// - 0 ... header area.
    /// - 1 ... watch area.
    /// - 2 ... history area.
    pub area_size: [tui::layout::Rect; 3],

    pub watch_area: WatchArea<'a>,

    results: Mutex<Vec<exec::Result>>,

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

        self.area_size = [top_chunks[0], main_chanks[0], main_chanks[1]];

        self.watch_area.set_area(self.area_size[1]);
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let block = Block::default().title("header");
        f.render_widget(block, self.area_size[0]);

        let mut text = "123\n456\n789";
        self.watch_area.update_data(text);
        self.watch_area.draw(f);

        let block = Block::default().title("history");
        f.render_widget(block, self.area_size[2]);
    }
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

    run_app(&mut terminal, &mut app);

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App) {
    // get Area Size from terminal.Frame
    let mut frame = terminal.get_frame();
    app.get_area(&mut frame);

    loop {
        // draw
        terminal.draw(|f| draw(f, &mut app));
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    app.draw(f)
}
