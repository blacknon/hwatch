// Copyright (c) 2022 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: batch modeを処理するのはmain.rsではなく、こっち側でやらせるのが良さそう?

use crossbeam_channel::{Receiver, Sender};
// module
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind, KeyEventState};
use regex::Regex;
use std::{collections::HashMap, io};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

// use std::process::Command;
use std::thread;

// local module
use crate::event::AppEvent;
use crate::exec::{exec_after_command, CommandResult};
use crate::header::HeaderArea;
use crate::help::HelpWindow;
use crate::history::{History, HistoryArea};
use crate::output;
use crate::watch::WatchArea;
use crate::{common::logging_result, Interval};

// local const
use crate::HISTORY_WIDTH;

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveArea {
    Watch,
    History,
}

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveWindow {
    Normal,
    Help,
}

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiffMode {
    Disable,
    Watch,
    Line,
    Word,
}

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Output,
    Stdout,
    Stderr,
}

///
#[derive(Clone, Copy, PartialEq, Eq)]
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
    after_command: String,

    ///
    ansi_color: bool,

    ///
    line_number: bool,

    ///
    is_beep: bool,

    ///
    is_filtered: bool,

    ///
    is_regex_filter: bool,

    ///
    show_history: bool,

    ///
    show_header: bool,

    ///
    filtered_text: String,

    ///
    input_mode: InputMode,

    ///
    output_mode: OutputMode,

    ///
    diff_mode: DiffMode,

    ///
    is_only_diffline: bool,

    ///
    results: HashMap<usize, CommandResult>,

    ///
    interval: Interval,

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
    pub fn new(tx: Sender<AppEvent>, rx: Receiver<AppEvent>, interval: Interval) -> Self {
        // method at create new view trail.
        Self {
            area: ActiveArea::History,
            window: ActiveWindow::Normal,

            after_command: "".to_string(),
            ansi_color: false,
            line_number: false,
            show_history: true,
            show_header: true,

            is_beep: false,
            is_filtered: false,
            is_regex_filter: false,
            filtered_text: "".to_string(),

            input_mode: InputMode::None,
            output_mode: OutputMode::Output,
            diff_mode: DiffMode::Disable,
            is_only_diffline: false,

            results: HashMap::new(),
            interval: interval.clone(),

            header_area: HeaderArea::new(*interval.read().unwrap()),
            history_area: HistoryArea::new(),
            watch_area: WatchArea::new(),

            help_window: HelpWindow::new(),

            done: false,
            logfile: "".to_string(),
            tx,
            rx,
        }
    }

    ///
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        self.history_area.next();
        let mut update_draw = true;
        loop {
            if self.done {
                return Ok(());
            }

            // Draw data
            if update_draw {
                terminal.draw(|f| self.draw(f))?;
                update_draw = false
            }

            // get event
            match self.rx.recv() {
                Ok(AppEvent::Redraw) => update_draw = true,

                // Get terminal event.
                Ok(AppEvent::TerminalEvent(terminal_event)) => {
                    self.get_event(terminal_event);
                    update_draw = true;
                }

                // Get command result.
                Ok(AppEvent::OutputUpdate(exec_result)) => {
                    let _exec_return = self.update_result(exec_result);

                    // beep
                    if _exec_return && self.is_beep {
                        println!("\x07")
                    }

                    update_draw = true;
                }

                // get exit event
                Ok(AppEvent::Exit) => self.done = true,

                Err(_) => {}
            }
        }
    }

    ///
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.define_subareas(f.size());

        if self.show_header {
            // Draw header area.
            self.header_area.draw(f);
        }

        // Draw watch area.
        self.watch_area.draw(f);

        self.history_area
            .set_active(self.area == ActiveArea::History);
        self.history_area.draw(f);

        // match help mode
        if let ActiveWindow::Help = self.window {
            self.help_window.draw(f);
            return;
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
    fn define_subareas(&mut self, total_area: tui::layout::Rect) {
        let history_width: u16 = match self.show_history {
            true => HISTORY_WIDTH,
            false => match self.area == ActiveArea::History
                || self.history_area.get_state_select() != 0
            {
                true => 2,
                false => 0,
            },
        };

        let header_height: u16 = match self.show_header {
            true => 2,
            false => 0,
        };

        // get Area's chunks
        let top_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(header_height),
                    Constraint::Max(total_area.height - header_height),
                ]
                .as_ref(),
            )
            .split(total_area);
        self.header_area.set_area(top_chunks[0]);

        let main_chunks = Layout::default()
            .constraints(
                [
                    Constraint::Max(total_area.width - history_width),
                    Constraint::Length(history_width),
                ]
                .as_ref(),
            )
            .direction(Direction::Horizontal)
            .split(top_chunks[1]);

        self.watch_area.set_area(main_chunks[0]);
        self.history_area.set_area(main_chunks[1]);
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
        // let results = self.results;

        // check result size.
        //　If the size of result is not 0 or more, return and not process.
        if self.results.is_empty() {
            return;
        }

        // text_src ... old text.
        let text_src: &str;

        // set target number at new history.
        let mut target_dst: usize = num;

        // check results over target...
        if target_dst == 0 {
            target_dst = self.results.len() - 1;
        }

        // set target number at old history.
        let target_src = target_dst - 1;

        // set new text(text_dst)
        let text_dst = match self.output_mode {
            OutputMode::Output => &self.results[&target_dst].output,
            OutputMode::Stdout => &self.results[&target_dst].stdout,
            OutputMode::Stderr => &self.results[&target_dst].stderr,
        };

        // set old text(text_src)
        if self.results.len() > target_src {
            match self.output_mode {
                OutputMode::Output => text_src = &self.results[&target_src].output,
                OutputMode::Stdout => text_src = &self.results[&target_src].stdout,
                OutputMode::Stderr => text_src = &self.results[&target_src].stderr,
            }
        } else {
            text_src = "";
        }

        let output_data = match self.diff_mode {
            DiffMode::Disable => output::get_plane_output(
                self.ansi_color,
                self.line_number,
                text_dst,
                self.is_filtered,
                self.is_regex_filter,
                &self.filtered_text,
            ),

            DiffMode::Watch => {
                output::get_watch_diff(self.ansi_color, self.line_number, text_src, text_dst)
            }

            DiffMode::Line => output::get_line_diff(
                self.ansi_color,
                self.line_number,
                self.is_only_diffline,
                text_src,
                text_dst,
            ),

            DiffMode::Word => output::get_word_diff(
                self.ansi_color,
                self.line_number,
                self.is_only_diffline,
                text_src,
                text_dst,
            ),
        };

        self.watch_area.update_output(output_data);
    }

    ///
    pub fn set_after_command(&mut self, command: String) {
        self.after_command = command;
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

    ///
    pub fn set_beep(&mut self, beep: bool) {
        self.is_beep = beep;
    }

    ///
    pub fn set_line_number(&mut self, line_number: bool) {
        self.line_number = line_number;

        self.header_area.set_line_number(line_number);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_interval(&mut self, interval: f64) {
        let mut cur_interval = self.interval.write().unwrap();
        *cur_interval = interval;
        self.header_area.set_interval(*cur_interval);
        self.header_area.update();
    }

    ///
    fn increase_interval(&mut self) {
        let cur_interval = *self.interval.read().unwrap();
        self.set_interval(cur_interval + 0.5);
    }

    ///
    fn decrease_interval(&mut self) {
        let cur_interval = *self.interval.read().unwrap();
        if cur_interval > 0.5 {
            self.set_interval(cur_interval - 0.5);
        }
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
    fn set_is_only_diffline(&mut self, is_only_diffline: bool) {
        self.is_only_diffline = is_only_diffline;

        self.header_area.set_is_only_diffline(is_only_diffline);
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

        self.header_area.set_input_mode(self.input_mode);
        self.header_area.update();
    }

    ///
    fn reset_history(&mut self, is_regex: bool) {
        // unlock self.results
        // let results = self.results;
        let counter = self.results.len();
        let mut tmp_history = vec![];

        // append result.
        let latest_num: usize = if counter > 1 { counter - 1 } else { 0 };

        tmp_history.push(History {
            timestamp: "latest                 ".to_string(),
            status: self.results[&latest_num].status,
            num: 0,
        });

        for result in self.results.clone().into_iter() {
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
                } else if !result_text.contains(&self.filtered_text) {
                    is_push = false;
                }
            }

            if is_push {
                tmp_history.push(History {
                    timestamp: result.1.timestamp.clone(),
                    status: result.1.status,
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
    fn update_result(&mut self, _result: CommandResult) -> bool {
        // check results size.
        let mut latest_result = CommandResult::default();

        if self.results.is_empty() {
            // diff output data.
            self.results.insert(0, latest_result.clone());
        } else {
            let latest_num = self.results.len() - 1;
            latest_result = self.results[&latest_num].clone();
        }

        // update HeaderArea
        self.header_area.set_current_result(_result.clone());
        self.header_area.update();

        // check result diff
        if latest_result == _result {
            return false;
        }

        if self.after_command != "".to_string() {
            let after_command = self.after_command.clone();

            let results = self.results.clone();
            let latest_num = results.len() - 1;

            let before_result = results[&latest_num].clone();
            let after_result = _result.clone();

            {
                thread::spawn(move || {
                    exec_after_command(
                        "sh -c".to_string(),
                        after_command.clone(),
                        before_result,
                        after_result,
                    );
                });
            }
        }

        // append results
        let result_index = self.results.len();
        self.results.insert(result_index, _result);

        // logging result.
        if !self.logfile.is_empty() {
            let _ = logging_result(&self.logfile, &self.results[&result_index]);
        }

        // update HistoryArea
        let mut selected = self.history_area.get_state_select();
        let mut is_push = true;
        if self.is_filtered {
            let result_text = &self.results[&result_index].output.clone();

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
            let _timestamp = &self.results[&result_index].timestamp;
            let _status = &self.results[&result_index].status;
            self.history_area
                .update(_timestamp.to_string(), *_status, result_index as u16);

            // update selected
            if selected != 0 {
                self.history_area.previous();
            }
        }
        selected = self.history_area.get_state_select();

        // update WatchArea
        self.set_output_data(selected);

        true
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
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.input_key_up(),

                    // down
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.input_key_down(),

                    // mouse wheel up
                    Event::Mouse(MouseEvent {
                        kind: MouseEventKind::ScrollUp,
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => self.mouse_scroll_up(),

                    // mouse wheel down
                    Event::Mouse(MouseEvent {
                        kind: MouseEventKind::ScrollDown,
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => self.mouse_scroll_down(),

                    Event::Mouse(MouseEvent {
                        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
                        column,
                        row,
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => {
                        // Currently a no-op
                        self.mouse_click_left(column, row);
                    }

                    // pgup

                    // pgdn

                    // left
                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.input_key_left(),

                    // right
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.input_key_right(),

                    // c
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_ansi_color(!self.ansi_color),

                    // d
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.toggle_diff_mode(),

                    // n
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('n'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_line_number(!self.line_number),

                    // o(lower o)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('o'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.toggle_output(),

                    // O(upper o). shift + o
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('O'),
                        modifiers: KeyModifiers::SHIFT,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_is_only_diffline(!self.is_only_diffline),

                    // 0 (DiffMode::Disable)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('0'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_diff_mode(DiffMode::Disable),

                    // 1 (DiffMode::Watch)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('1'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_diff_mode(DiffMode::Watch),

                    // 2 (DiffMode::Line)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('2'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_diff_mode(DiffMode::Line),

                    // 3 (DiffMode::Word)
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('3'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_diff_mode(DiffMode::Word),

                    // F1 (OutputMode::Stdout)
                    Event::Key(KeyEvent {
                        code: KeyCode::F(1),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_output_mode(OutputMode::Stdout),

                    // F2 (OutputMode::Stderr)
                    Event::Key(KeyEvent {
                        code: KeyCode::F(2),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_output_mode(OutputMode::Stderr),

                    // F3 (OutputMode::Output)
                    Event::Key(KeyEvent {
                        code: KeyCode::F(3),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_output_mode(OutputMode::Output),

                    // + Increase interval
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('+'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.increase_interval(),

                    // - Decrease interval
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('-'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.decrease_interval(),

                    // Tab ... Toggle Area(Watch or History).
                    Event::Key(KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.toggle_area(),

                    // / ... Change Filter Mode(plane text).
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('/'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_input_mode(InputMode::Filter),

                    // / ... Change Filter Mode(regex text).
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('*'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.set_input_mode(InputMode::RegexFilter),

                    // ESC ... Reset.
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
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
                    // Backspace ... toggle history panel.
                    Event::Key(KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.show_history(!self.show_history),

                    // Common input key
                    // t ... toggle ui
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('t'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.show_ui(!self.show_header),

                    // Common input key
                    // h ... toggle help window.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.toggle_window(),

                    // q ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self
                        .tx
                        .send(AppEvent::Exit)
                        .expect("send error hwatch exit."),

                    // Ctrl + C ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
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
                    // up
                    Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.input_key_up(),

                    // down
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.input_key_down(),

                    // h ... toggle help window.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self.toggle_window(),

                    // q ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => self
                        .tx
                        .send(AppEvent::Exit)
                        .expect("send error hwatch exit."),

                    // Ctrl + C ... exit hwatch.
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
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
        if let Event::Key(key) = terminal_event {
            match key.code {
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
                    // check regex error...
                    if is_regex {
                        let input_text = self.header_area.input_text.clone();
                        let re_result = Regex::new(&input_text);
                        if re_result.is_err() {
                            // TODO: create print message method.
                            return;
                        }
                    }

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
            }
        }
    }

    fn set_area(&mut self, target: ActiveArea) {
        self.area = target;
        // set active window to header.
        self.header_area.set_active_area(self.area);
        self.header_area.update();
    }

    ///
    fn toggle_area(&mut self) {
        if let ActiveWindow::Normal = self.window {
            match self.area {
                ActiveArea::Watch => self.set_area(ActiveArea::History),
                ActiveArea::History => self.set_area(ActiveArea::Watch),
            }
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
    pub fn show_history(&mut self, visible: bool) {
        self.show_history = visible;
        if !visible {
            self.set_area(ActiveArea::Watch);
        }
        let _ = self.tx.send(AppEvent::Redraw);
    }

    ///
    pub fn show_ui(&mut self, visible: bool) {
        self.show_header = visible;
        self.show_history = visible;
        let _ = self.tx.send(AppEvent::Redraw);
    }

    ///
    pub fn show_help_banner(&mut self, visible: bool) {
        self.header_area.set_banner(
            if visible {
                "Display help with h key!"
            } else {
                ""
            }
            .to_string(),
        );
        let _ = self.tx.send(AppEvent::Redraw);
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

    // Mouse wheel always scrolls the main area
    fn mouse_scroll_up(&mut self) {
        self.watch_area.scroll_up(2);
    }

    fn mouse_scroll_down(&mut self) {
        self.watch_area.scroll_down(2);
    }

    // NOTE: TODO:
    // Not currently used.
    // It will not be supported until the following issues are resolved.
    //     - https://github.com/tui-rs-revival/ratatui/pull/12
    ///
    //fn input_key_pgup(&mut self) {}

    // NOTE: TODO:
    // Not currently used.
    // It will not be supported until the following issues are resolved.
    //     - https://github.com/tui-rs-revival/ratatui/pull/12
    ///
    //fn input_key_pgdn(&mut self) {}

    ///
    fn input_key_left(&mut self) {
        if let ActiveWindow::Normal = self.window {
            self.area = ActiveArea::Watch;

            // set active window to header.
            self.header_area.set_active_area(self.area);
            self.header_area.update();
        }
    }

    ///
    fn input_key_right(&mut self) {
        if let ActiveWindow::Normal = self.window {
            self.area = ActiveArea::History;

            // set active window to header.
            self.header_area.set_active_area(self.area);
            self.header_area.update();
        }
    }

    // NOTE: TODO: Currently does not do anything
    // Mouse clicks will not be supported until the following issues are resolved.
    //     - https://github.com/tui-rs-revival/ratatui/pull/12
    fn mouse_click_left(&mut self, _column: u16, _row: u16) {
        //    // check in hisotry area
        //    let is_history_area = check_in_area(self.history_area.area, column, row);
        //    if is_history_area {
        //        // let headline_count = self.history_area.area.y;
        //        // self.history_area.click_row(row - headline_count);

        //        // self.history_area.previous();

        //        let selected = self.history_area.get_state_select();
        //        self.set_output_data(selected);
        //    }
    }
}

// NOTE: TODO:
// Not currently used.
//     - https://github.com/tui-rs-revival/ratatui/pull/12
//fn check_in_area(area: Rect, column: u16, row: u16) -> bool {
//    let mut result = true;
//
//    // get area range's
//    let area_top = area.top();
//    let area_bottom = area.bottom();
//    let area_left = area.left();
//    let area_right = area.right();
//
//    let area_row_range = area_top..area_bottom;
//    let area_column_range = area_left..area_right;
//
//    if !area_row_range.contains(&row) {
//        result = false;
//    }
//
//    if !area_column_range.contains(&column) {
//        result = false;
//    }
//
//    result
//}
