// Copyright (c) 2021 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[warn(unused_doc_comments)]
// module
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use regex::Regex;
use std::{
    collections::HashMap,
    io,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::Duration,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::Spans,
    Frame, Terminal,
};

// local module
use common::{differences_result, logging_result};
use event::AppEvent;
use exec::CommandResult;
use header::HeaderArea;
use help::HelpWindow;
use history::{History, HistoryArea};
use output;
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
pub enum InputMode {
    None,
    Filter,
    RegexFilter,
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
    line_number: bool,

    ///
    is_filtered: bool,

    ///
    is_regex_filter: bool,

    ///
    filtered_text: String,

    ///
    input_mode: InputMode,

    ///
    output_mode: OutputMode,

    ///
    diff_mode: DiffMode,

    ///
    results: Mutex<HashMap<usize, CommandResult>>,

    ///
    header_area: HeaderArea<'a>,

    ///
    history_area: HistoryArea,

    ///
    watch_area: WatchArea<'a>,

    ///
    help_window: HelpWindow<'a>,

    /// It is a flag value to confirm the done of the app.
    /// If `true`, exit app.
    pub done: bool,

    /// logfile path.
    logfile: String,

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
            line_number: false,

            is_filtered: false,
            is_regex_filter: false,
            filtered_text: "".to_string(),

            input_mode: InputMode::None,
            output_mode: OutputMode::Output,
            diff_mode: DiffMode::Disable,

            results: Mutex::new(HashMap::new()),

            header_area: HeaderArea::new(),
            history_area: HistoryArea::new(),
            watch_area: WatchArea::new(),

            help_window: HelpWindow::new(),

            done: false,
            logfile: "".to_string(),
            tx: tx,
            rx: rx,
        }
    }

    ///
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

    ///
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.get_area(f);

        // Draw header area.
        self.header_area.draw(f);

        // Draw watch area.
        self.watch_area.draw(f);

        // Draw history area
        self.history_area.draw(f);

        // match help mode
        match self.window {
            ActiveWindow::Help => {
                self.help_window.draw(f);
                return;
            }
            _ => {}
        }

        // match input_mode
        match self.input_mode {
            InputMode::Filter | InputMode::RegexFilter => {
                //
                let input_text_x = self.header_area.input_text.len() as u16 + 1;
                let input_text_y = self.header_area.area.y + 1;

                // set cursor
                f.set_cursor(input_text_x, input_text_y);
            }

            _ => {}
        }
    }

    ///
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

    ///
    fn get_event(&mut self, terminal_event: crossterm::event::Event) {
        match self.input_mode {
            InputMode::None => self.get_normal_input_key(terminal_event),
            InputMode::Filter => self.get_filter_input_key(false, terminal_event),
            InputMode::RegexFilter => self.get_filter_input_key(true, terminal_event),
        }
    }

    /// Set the history to be output to WatchArea.
    fn set_output_data(&mut self, num: usize) {
        let results = self.results.lock().unwrap();

        // text_src ... old text.
        // text_dst ... new text.
        let text_src: &str;
        let text_dst: &str;
        let mut output_data = vec![];

        // set target number at new history.
        let mut target_dst: usize = num;

        // check results over target...
        if target_dst == 0 {
            target_dst = results.len() - 1;
        }

        // set target number at old history.
        let target_src = target_dst - 1;

        // set new text(text_dst)
        match self.output_mode {
            OutputMode::Output => text_dst = &results[&target_dst].output,
            OutputMode::Stdout => text_dst = &results[&target_dst].stdout,
            OutputMode::Stderr => text_dst = &results[&target_dst].stderr,
        }

        // set old text(text_src)
        if results.len() > target_src {
            match self.output_mode {
                OutputMode::Output => text_src = &results[&target_src].output,
                OutputMode::Stdout => text_src = &results[&target_src].stdout,
                OutputMode::Stderr => text_src = &results[&target_src].stderr,
            }
        } else {
            text_src = "";
        }

        match self.diff_mode {
            DiffMode::Disable => {
                output_data = output::get_plane_output(
                    self.ansi_color,
                    self.line_number,
                    &text_dst,
                    self.is_filtered,
                    self.is_regex_filter,
                    &self.filtered_text,
                );
            }

            DiffMode::Watch => {
                output_data =
                    output::get_watch_diff(self.ansi_color, self.line_number, &text_src, &text_dst);
            }

            DiffMode::Line => {
                output_data =
                    output::get_line_diff(self.ansi_color, self.line_number, &text_src, &text_dst);
            }

            DiffMode::Word => {
                output_data =
                    output::get_word_diff(self.ansi_color, self.line_number, &text_src, &text_dst);
            }
        }

        self.watch_area.update_output(output_data);
    }

    ///
    pub fn set_output_mode(&mut self, mode: OutputMode) {
        self.output_mode = mode;
        self.header_area.set_output_mode(mode);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_ansi_color(&mut self, ansi_color: bool) {
        self.ansi_color = ansi_color;

        self.header_area.set_ansi_color(ansi_color);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    pub fn set_line_number(&mut self, line_number: bool) {
        self.line_number = line_number;

        self.header_area.set_line_number(line_number);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_interval(&mut self, interval: f64) {
        self.header_area.set_interval(interval);
    }

    ///
    pub fn set_diff_mode(&mut self, diff_mode: DiffMode) {
        self.diff_mode = diff_mode;

        self.header_area.set_diff_mode(diff_mode);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_logpath(&mut self, logpath: String) {
        self.logfile = logpath;
    }

    ///
    fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;

        self.header_area.set_input_mode(self.input_mode.clone());
        self.header_area.update();
    }

    ///
    fn reset_history(&mut self, is_regex: bool) {
        // unlock self.results
        let results = self.results.lock().unwrap();
        let counter = results.len();
        let mut tmp_history = vec![];

        // append result.
        let latest_num = counter - 1;
        tmp_history.push(History {
            timestamp: "latest                 ".to_string(),
            status: results[&latest_num].status.clone(),
            num: 0 as u16,
        });

        for result in results.clone().into_iter() {
            if result.0 == 0 {
                continue;
            }

            let mut is_push = true;
            if self.is_filtered {
                let result_text = &result.1.output.clone();

                if is_regex {
                    let re = Regex::new(&self.filtered_text.clone()).unwrap();
                    let regex_match = re.is_match(result_text);
                    if !regex_match {
                        is_push = false;
                    }
                } else {
                    if !result_text.contains(&self.filtered_text) {
                        is_push = false;
                    }
                }
            }

            if is_push {
                tmp_history.push(History {
                    timestamp: result.1.timestamp.clone(),
                    status: result.1.status.clone(),
                    num: result.0 as u16,
                });
            }
        }

        // sort tmp_history, to push history
        let mut history = vec![];
        tmp_history.sort_by(|a, b| b.num.cmp(&a.num));

        for h in tmp_history.into_iter() {
            if h.num == 0 {
                history.insert(0, vec![h]);
            } else {
                history.push(vec![h]);
            }
        }

        // reset data.
        self.history_area.reset_history_data(history);
    }

    ///
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

        // set tmp result
        latest_result = &tmp_result;
        if results.len() == 0 {
            // diff output data.
            results.insert(0, tmp_result.clone());
        } else {
            let latest_num = results.len() - 1;
            latest_result = &results[&latest_num];
        }

        // update HeaderArea
        self.header_area.set_current_result(_result.clone());
        self.header_area.update();

        // check result diff
        let check_result_diff = differences_result(latest_result, &_result);
        if check_result_diff {
            return;
        }

        // append results
        let result_index = results.len();
        results.insert(result_index, _result.clone());

        // logging result.
        if self.logfile != "" {
            let _ = logging_result(&self.logfile, &results[&result_index]);
        }

        // update HistoryArea
        let mut selected = self.history_area.get_state_select();
        let mut is_push = true;
        if self.is_filtered {
            let result_text = &results[&result_index].output.clone();

            match self.is_regex_filter {
                true => {
                    let re = Regex::new(&self.filtered_text.clone()).unwrap();
                    let regex_match = re.is_match(result_text);
                    if !regex_match {
                        is_push = false;
                    }
                }

                false => {
                    if !result_text.contains(&self.filtered_text) {
                        is_push = false;
                    }
                }
            }
        }
        if is_push {
            let _timestamp = &results[&result_index].timestamp;
            let _status = &results[&result_index].status;
            self.history_area
                .update(_timestamp.to_string(), _status.clone(), result_index as u16);

            // update selected
            if selected != 0 {
                self.history_area.previous();
            }
        }
        selected = self.history_area.get_state_select();

        // drop mutex
        drop(results);

        // update WatchArea
        self.set_output_data(selected)
    }

    ///
    fn get_normal_input_key(&mut self, terminal_event: crossterm::event::Event) {
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
                    }) => self.set_ansi_color(!self.ansi_color),

                    // d
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_diff_mode(),

                    // n
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('n'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_line_number(!self.line_number),

                    // o
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('o'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_output(),

                    // 0 (DiffMode::Disable)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('0'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_diff_mode(DiffMode::Disable),

                    // 1 (DiffMode::Watch)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('1'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_diff_mode(DiffMode::Watch),

                    // 2 (DiffMode::Line)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('2'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_diff_mode(DiffMode::Line),

                    // 3 (DiffMode::Word)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('3'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_diff_mode(DiffMode::Word),

                    // F1 (OutputMode::Stdout)
                    Event::Key(KeyEvent {
                        code: KeyCode::F(1),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_output_mode(OutputMode::Stdout),

                    // F2 (OutputMode::Stderr)
                    Event::Key(KeyEvent {
                        code: KeyCode::F(2),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_output_mode(OutputMode::Stderr),

                    // F3 (OutputMode::Output)
                    Event::Key(KeyEvent {
                        code: KeyCode::F(3),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_output_mode(OutputMode::Output),

                    // Tab ... Toggle Area(Watch or History).
                    Event::Key(KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_area(),

                    // / ... Change Filter Mode(plane text).
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('/'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_input_mode(InputMode::Filter),

                    // / ... Change Filter Mode(regex text).
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('*'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.set_input_mode(InputMode::RegexFilter),

                    // ESC ... Reset.
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                    }) => {
                        self.is_filtered = false;
                        self.is_regex_filter = false;
                        self.filtered_text = "".to_string();
                        self.header_area.input_text = self.filtered_text.clone();
                        self.set_input_mode(InputMode::None);
                        self.reset_history(false);

                        let selected = self.history_area.get_state_select();

                        // update WatchArea
                        self.set_output_data(selected);
                    }

                    // Common input key
                    // h ... toggel help window.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_window(),

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

                    // mouse event.
                    // Event::Mouse(ref mouse_event) => self.get_input_mouse_event(mouse_event),
                    _ => {}
                }
            }
            ActiveWindow::Help => {
                match terminal_event {
                    // Common input key
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

                    // h ... toggel help window.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::NONE,
                    }) => self.toggle_window(),

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

    ///
    fn get_filter_input_key(&mut self, is_regex: bool, terminal_event: crossterm::event::Event) {
        match terminal_event {
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => {
                    // add header input_text;
                    self.header_area.input_text.push(c);
                    self.header_area.update();
                }

                KeyCode::Backspace => {
                    // remove header input_text;
                    self.header_area.input_text.pop();
                    self.header_area.update();
                }

                KeyCode::Enter => {
                    // set filtered mode enable
                    self.is_filtered = true;
                    self.is_regex_filter = is_regex;
                    self.filtered_text = self.header_area.input_text.clone();
                    self.set_input_mode(InputMode::None);
                    self.reset_history(is_regex);

                    let selected = self.history_area.get_state_select();

                    // update WatchArea
                    self.set_output_data(selected);
                }

                KeyCode::Esc => {
                    self.header_area.input_text = self.filtered_text.clone();
                    self.set_input_mode(InputMode::None);
                    self.reset_history(is_regex);

                    let selected = self.history_area.get_state_select();

                    // update WatchArea
                    self.set_output_data(selected);
                }

                _ => {}
            },

            _ => {}
        }
    }

    // Not currently used.
    ///
    fn get_input_mouse_event(&mut self, mouse_event: &MouseEvent) {
        let mouse_event_tupple = (mouse_event.kind, mouse_event.modifiers);
        match mouse_event_tupple {
            // Click Mouse Left.
            (MouseEventKind::Down(MouseButton::Left), KeyModifiers::NONE) => {
                self.mouse_click_left(mouse_event.column, mouse_event.row);
            }

            _ => {}
        }
    }

    ///
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

    ///
    fn toggle_output(&mut self) {
        match self.output_mode {
            OutputMode::Output => self.set_output_mode(OutputMode::Stdout),
            OutputMode::Stdout => self.set_output_mode(OutputMode::Stderr),
            OutputMode::Stderr => self.set_output_mode(OutputMode::Output),
        }
    }

    ///
    fn toggle_diff_mode(&mut self) {
        match self.diff_mode {
            DiffMode::Disable => self.set_diff_mode(DiffMode::Watch),
            DiffMode::Watch => self.set_diff_mode(DiffMode::Line),
            DiffMode::Line => self.set_diff_mode(DiffMode::Word),
            DiffMode::Word => self.set_diff_mode(DiffMode::Disable),
        }
    }

    ///
    fn toggle_window(&mut self) {
        match self.window {
            ActiveWindow::Normal => self.window = ActiveWindow::Help,
            ActiveWindow::Help => self.window = ActiveWindow::Normal,
        }
    }

    ///
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
            ActiveWindow::Help => {
                self.help_window.scroll_up(1);
            }
        }
    }

    ///
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
            ActiveWindow::Help => {
                self.help_window.scroll_down(1);
            }
        }
    }

    // NOTE: TODO:
    // Not currently used.
    // It will not be supported until the following issues are resolved.
    //     - https://github.com/fdehau/tui-rs/issues/495
    ///
    fn input_key_pgup(&mut self) {}

    // NOTE: TODO:
    // Not currently used.
    // It will not be supported until the following issues are resolved.
    //     - https://github.com/fdehau/tui-rs/issues/495
    ///
    fn input_key_pgdn(&mut self) {}

    ///
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

    ///
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

    // NOTE: TODO:
    // Not currently used.
    // It will not be supported until the following issues are resolved.
    //     - https://github.com/fdehau/tui-rs/issues/495
    ///
    fn mouse_click_left(&mut self, column: u16, row: u16) {
        // check in hisotry area
        let is_history_area = check_in_area(self.history_area.area, column, row);
        if is_history_area {
            let headline_count = self.history_area.area.y;
            // self.history_area.click_row(row - headline_count);

            // self.history_area.previous();

            let selected = self.history_area.get_state_select();
            self.set_output_data(selected);
        }
    }

    ///
    fn mouse_scroll_up(&mut self, column: u16, row: u16) {}

    ///
    fn mouse_scroll_down(&mut self, column: u16, row: u16) {}
}

///
fn check_in_area(area: Rect, column: u16, row: u16) -> bool {
    let mut result = true;

    // get area range's
    let area_top = area.top();
    let area_bottom = area.bottom();
    let area_left = area.left();
    let area_right = area.right();

    let area_row_range = area_top..area_bottom;
    let area_column_range = area_left..area_right;

    if !area_row_range.contains(&row) {
        result = false;
    }

    if !area_column_range.contains(&column) {
        result = false;
    }

    return result;
}
