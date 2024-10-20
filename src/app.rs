// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: historyの一個前、をdiffで取れるようにする(今は問答無用でVecの1個前のデータを取得しているから、ちょっと違う方法を取る?)

// module
use crossbeam_channel::{Receiver, Sender};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEvent
    },
    execute,
};
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, Write},
    rc::Rc,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Position},
    Frame, Terminal,
};
use std::thread;

// local module
use crate::{common::{logging_result, DiffMode, OutputMode}, keymap::InputEventContents};
use crate::event::AppEvent;
use crate::exec::{exec_after_command, CommandResult};
use crate::exit::ExitWindow;
use crate::header::HeaderArea;
use crate::help::HelpWindow;
use crate::history::{History, HistorySummary, HistoryArea};
use crate::keymap::{Keymap, default_keymap, InputAction};
use crate::output;
use crate::watch::WatchArea;

// local const
use crate::HISTORY_WIDTH;
use crate::DEFAULT_TAB_SIZE;
use crate::Interval;

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
    Exit,
}

///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    Filter,
    RegexFilter,
}

#[derive(Clone)]
/// Struct to hold history summary and CommandResult set.
/// Since the calculation source of the history summary changes depending on the output mode, it is necessary to set it separately from the command result.
struct ResultItems {
    pub command_result: Rc<CommandResult>,
    pub summary: HistorySummary,
}

/// Struct at watch view window.
pub struct App<'a> {
    ///
    keymap: Keymap,

    ///
    area: ActiveArea,

    ///
    window: ActiveWindow,

    ///
    after_command: String,

    ///
    limit: u32,

    ///
    ansi_color: bool,

    ///
    line_number: bool,

    ///
    reverse: bool,

    ///
    disable_exit_dialog: bool,

    ///
    is_beep: bool,

    ///
    is_border: bool,

    ///
    is_history_summary: bool,

    ///
    is_scroll_bar: bool,

    /// If text search filtering is enabled.
    is_filtered: bool,

    /// If regex search filtering is enabled.
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

    /// result at output.
    /// Use the same value as the key usize for results, results_stdout, and results_stderr, and use it as the key when switching outputs.
    results: HashMap<usize, ResultItems>,

    // @TODO: resultsのメモリ位置を参照させるように変更
    /// result at output only stdout.
    /// Use the same value as the key usize for results, results_stdout, and results_stderr, and use it as the key when switching outputs.
    results_stdout: HashMap<usize, ResultItems>,

    // @TODO: resultsのメモリ位置を参照させるように変更
    /// result at output only stderr.
    /// Use the same value as the key usize for results, results_stdout, and results_stderr, and use it as the key when switching outputs.
    results_stderr: HashMap<usize, ResultItems>,

    ///
    enable_summary_char: bool,

    ///
    interval: Interval,

    ///
    tab_size: u16,

    ///
    header_area: HeaderArea<'a>,

    ///
    history_area: HistoryArea,

    ///
    watch_area: WatchArea<'a>,

    ///
    help_window: HelpWindow<'a>,

    ///
    exit_window: ExitWindow<'a>,

    /// Enable mouse wheel support.
    mouse_events: bool,

    ///
    printer: output::Printer,

    /// It is a flag value to confirm the done of the app.
    /// If `true`, exit app.
    pub done: bool,

    /// logfile path.
    logfile: String,

    ///
    pub tx: Sender<AppEvent>,

    ///
    pub rx: Receiver<AppEvent>,
}

/// Trail at watch view window.
impl<'a> App<'a> {
    ///
    pub fn new(
        tx: Sender<AppEvent>,
        rx: Receiver<AppEvent>,
        interval: Interval,
    ) -> Self {
        // method at create new view trail.
        Self {
            keymap: default_keymap(),

            area: ActiveArea::History,
            window: ActiveWindow::Normal,

            limit: 0,

            after_command: "".to_string(),
            ansi_color: false,
            line_number: false,
            reverse: false,
            disable_exit_dialog: false,
            show_history: true,
            show_header: true,

            is_beep: false,
            is_border: false,
            is_history_summary: false,
            is_scroll_bar: false,
            is_filtered: false,
            is_regex_filter: false,
            filtered_text: "".to_string(),

            input_mode: InputMode::None,
            output_mode: OutputMode::Output,
            diff_mode: DiffMode::Disable,
            is_only_diffline: false,

            results: HashMap::new(),
            results_stdout: HashMap::new(),
            results_stderr: HashMap::new(),

            enable_summary_char: false,

            interval: interval.clone(),
            tab_size: DEFAULT_TAB_SIZE,

            header_area: HeaderArea::new(*interval.read().unwrap()),
            history_area: HistoryArea::new(),
            watch_area: WatchArea::new(),

            help_window: HelpWindow::new(default_keymap()),
            exit_window: ExitWindow::new(),

            mouse_events: false,

            printer: output::Printer::new(),

            done: false,
            logfile: "".to_string(),
            tx,
            rx,
        }
    }

    ///
    pub fn run<B: Backend + Write>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        // set history setting
        self.history_area.set_enable_char_diff(self.enable_summary_char);
        self.history_area.next(1);

        let mut update_draw = true;

        self.printer
            .set_batch(false)
            .set_color(self.ansi_color)
            .set_diff_mode(self.diff_mode)
            .set_line_number(self.line_number)
            .set_output_mode(self.output_mode)
            .set_tab_size(self.tab_size)
            .set_only_diffline(self.is_only_diffline);

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
                    // TODO: thread化し、`update_draw`をsignalで受け取るような仕組みにする
                    eprintln!("before update_result"); // Debug
                    let _exec_return = self.update_result(exec_result, true);
                    eprintln!("after update_result"); // Debug

                    // beep
                    if _exec_return && self.is_beep {
                        println!("\x07")
                    }

                    update_draw = true;
                }

                //
                Ok(AppEvent::ChangeFlagMouseEvent) => {
                    if self.mouse_events {
                        execute!(terminal.backend_mut(), EnableMouseCapture)?;
                    } else {
                        execute!(terminal.backend_mut(), DisableMouseCapture)?;
                    }
                }

                // get exit event
                Ok(AppEvent::Exit) => self.done = true,

                Err(_) => {}
            }

            if update_draw {
                self.watch_area.update_wrap();
            }
        }
    }

    ///
    pub fn draw(&mut self, f: &mut Frame) {
        self.define_subareas(f.area());

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
        }

        if let ActiveWindow::Exit = self.window {
            self.exit_window.draw(f);
        }

        if self.window != ActiveWindow::Normal {
            return;
        }

        // match input_mode
        match self.input_mode {
            InputMode::Filter | InputMode::RegexFilter => {
                //
                let input_text_x = self.header_area.input_text.len() as u16 + 1;
                let input_text_y = self.header_area.area.y + 1;

                // set cursor
                f.set_cursor_position(Position { x: input_text_x, y: input_text_y });
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

    ///
    fn get_input_action(&self, terminal_event: &crossterm::event::Event) -> Option<&InputEventContents> {
        match terminal_event {
            &Event::Key(_) => {
                return self.keymap.get(terminal_event);
            },
            &Event::Mouse(mouse) => {
                let mouse_event = MouseEvent {
                    kind: mouse.kind,
                    column: 0,
                    row: 0,
                    modifiers: KeyModifiers::empty(),
                };
                return self.keymap.get(&Event::Mouse(mouse_event));

            },
            _ => {
                return None;
            }
        }
    }

    /// Set the history to be output to WatchArea.
    fn set_output_data(&mut self, num: usize) {
        // Switch the result depending on the output mode.
        let results = match self.output_mode {
            OutputMode::Output => &self.results,
            OutputMode::Stdout => &self.results_stdout,
            OutputMode::Stderr => &self.results_stderr,
        };

        // check result size.
        //　If the size of result is not 0 or more, return and not process.
        if results.is_empty() {
            return;
        }

        // set target number at new history.
        let mut target_dst: usize = num;

        // check results over target...
        if target_dst == 0 {
            target_dst = get_results_latest_index(results);
        } else {
            target_dst = get_near_index(results, target_dst);
        }
        let previous_dst = get_results_previous_index(results, target_dst);

        // set new text(text_dst)
        let dest: &CommandResult = &results[&target_dst].command_result;

        // set old text(text_src)
        let mut src = dest;
        if previous_dst > 0 {
            src = &results[&previous_dst].command_result;
        }

        let output_data = self.printer.get_watch_text(dest, src);

        // TODO: output_dataのtabをスペース展開する処理を追加

        self.watch_area.is_line_number = self.line_number;
        self.watch_area.update_output(output_data);
    }

    ///
    pub fn set_keymap(&mut self,keymap: Keymap) {
        self.keymap = keymap.clone();
        self.help_window = HelpWindow::new(self.keymap.clone());
    }

    ///
    pub fn set_after_command(&mut self, command: String) {
        self.after_command = command;
    }

    ///
    pub fn set_output_mode(&mut self, mode: OutputMode) {
        // header update
        self.output_mode = mode;
        self.header_area.set_output_mode(mode);
        self.header_area.update();

        // set output mode
        self.printer.set_output_mode(mode);

        // set output data
        if self.results.len() > 0 {
            // Switch the result depending on the output mode.
            let results = match self.output_mode {
                OutputMode::Output => &self.results,
                OutputMode::Stdout => &self.results_stdout,
                OutputMode::Stderr => &self.results_stderr,
            };

            let selected: usize = self.history_area.get_state_select();
            let new_selected = get_near_index(&results, selected);
            self.reset_history(new_selected);
            self.set_output_data(new_selected);
        }
    }

    ///
    pub fn set_ansi_color(&mut self, ansi_color: bool) {
        self.ansi_color = ansi_color;

        self.header_area.set_ansi_color(ansi_color);
        self.header_area.update();

        self.printer.set_color(ansi_color);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_beep(&mut self, beep: bool) {
        self.is_beep = beep;
    }

    ///
    pub fn set_border(&mut self, border: bool) {
        self.is_border = border;

        // set border
        self.history_area.set_border(border);
        self.watch_area.set_border(border);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_limit(&mut self, limit: u32) {
        self.limit = limit;
    }

    ///
    pub fn set_history_summary(&mut self, history_summary: bool) {
        self.is_history_summary = history_summary;

        // set history_summary
        self.history_area.set_summary(history_summary);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_scroll_bar(&mut self, scroll_bar: bool) {
        self.is_scroll_bar = scroll_bar;

        // set scroll_bar
        self.history_area.set_scroll_bar(scroll_bar);
        self.watch_area.set_scroll_bar(scroll_bar);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_line_number(&mut self, line_number: bool) {
        self.line_number = line_number;

        self.header_area.set_line_number(line_number);
        self.header_area.update();

        self.printer.set_line_number(line_number);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;

        self.header_area.set_reverse(reverse);
        self.header_area.update();

        self.printer.set_reverse(reverse);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_tab_size(&mut self, tab_size: u16) {
        self.tab_size = tab_size;
        self.printer.set_tab_size(tab_size);
    }

    ///
    pub fn set_enable_summary_char(&mut self, enable_summary_char: bool) {
        self.enable_summary_char = enable_summary_char;
    }

    ///
    pub fn set_interval(&mut self, interval: f64) {
        let mut cur_interval = self.interval.write().unwrap();
        *cur_interval = interval;
        self.header_area.set_interval(*cur_interval);
        self.header_area.update();
    }

    ///
    pub fn set_mouse_events(&mut self, mouse_events: bool) {
        self.mouse_events = mouse_events;
        self.tx.send(AppEvent::ChangeFlagMouseEvent).unwrap();
    }

    ///
    pub fn add_results(&mut self, results: Vec<CommandResult>) {
        for result in results {
            self.update_result(result, false);
        }
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

        self.printer.set_diff_mode(diff_mode);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    pub fn set_is_only_diffline(&mut self, is_only_diffline: bool) {
        self.is_only_diffline = is_only_diffline;

        self.header_area.set_is_only_diffline(is_only_diffline);
        self.header_area.update();

        self.printer.set_only_diffline(is_only_diffline);

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
    fn reset_history(&mut self, selected: usize) {
        // Switch the result depending on the output mode.
        let results = match self.output_mode {
            OutputMode::Output => &self.results,
            OutputMode::Stdout => &self.results_stdout,
            OutputMode::Stderr => &self.results_stderr,
        };

        // append result.
        let mut tmp_history = vec![];
        let latest_num: usize = get_results_latest_index(&results);
        tmp_history.push(History {
            timestamp: "latest                 ".to_string(),
            status: results[&latest_num].command_result.status,
            num: 0,
            summary: HistorySummary::init(),
        });

        let mut new_select: Option<usize> = None;
        // let mut previous_result: String = "".to_string();
        let mut results_vec = results.iter().collect::<Vec<(&usize, &ResultItems)>>();
        results_vec.sort_by_key(|&(key, _)| key);

        for (key, result) in results_vec {
            if key == &0 {
                continue;
            }

            let mut is_push = true;
            if self.is_filtered {
                let result_text = match self.output_mode {
                    OutputMode::Output => result.command_result.get_output(),
                    OutputMode::Stdout => result.command_result.get_stdout(),
                    OutputMode::Stderr => result.command_result.get_stderr(),
                };

                match self.is_regex_filter {
                    true => {
                        let re = Regex::new(&self.filtered_text.clone()).unwrap();
                        let regex_match = re.is_match(&result_text);
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

            if &selected == key {
                new_select = Some(selected);
            }

            if is_push {
                tmp_history.push(History {
                    timestamp: result.command_result.timestamp.clone(),
                    status: result.command_result.status,
                    num: *key as u16,
                    summary: result.summary.clone(),
                });
            }
        }

        if new_select.is_none() {
            new_select = Some(get_near_index(&results, selected));
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

        // @TODO: selectedをうまいことやる

        // reset data.
        self.history_area.reset_history_data(history);
        self.history_area.set_state_select(new_select.unwrap());

    }

    ///
    fn update_result(&mut self, _result: CommandResult, is_running_app: bool) -> bool {
        // check results size.
        eprintln!("update_result: check results size"); // Debug
        let mut latest_result = ResultItems {
            command_result: Rc::new(CommandResult::default()),
            summary: HistorySummary::init(),
        };

        eprintln!("update_result: check results is empty"); // Debug
        if self.results.is_empty() {
            // diff output data.
            self.results.insert(0, latest_result.clone());
            self.results_stdout.insert(0, latest_result.clone());
            self.results_stderr.insert(0, latest_result.clone());
        } else {
            let latest_num = get_results_latest_index(&self.results);
            latest_result = self.results[&latest_num].clone();
        }

        // update HeaderArea
        eprintln!("update_result: update HeaderArea"); // Debug
        self.header_area.set_current_result(_result.clone());
        self.header_area.update();

        // check result diff
        // NOTE: ここで実行結果の差分を比較している // 0.3.12リリースしたら消す
        eprintln!("update_result: check result diff"); // Debug
        if latest_result.command_result == Rc::new(_result.clone()) {
            return false;
        }

        eprintln!("update_result: check after command"); // Debug
        if !self.after_command.is_empty() && is_running_app {
            let after_command = self.after_command.clone();

            let results = self.results.clone();
            let latest_num = results.len() - 1;

            let before_result:CommandResult = (*results[&latest_num].command_result).clone();
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
        eprintln!("update_result: append results"); // Debug
        let insert_result = self.insert_result(_result); eprintln!("update_result: append results: insert_result"); // Debug
        let result_index = insert_result.0;
        let is_limit_over = insert_result.1;
        let is_update_stdout = insert_result.2;
        let is_update_stderr = insert_result.3;

        // logging result.
        eprintln!("update_result: logging result"); // Debug
        if !self.logfile.is_empty() {
            let _ = logging_result(&self.logfile, &self.results[&result_index].command_result);
        }

        // update HistoryArea
        eprintln!("update_result: update HistoryArea"); // Debug
        let mut is_push = true;
        if self.is_filtered {
            let result_text = match self.output_mode {
                OutputMode::Output => self.results[&result_index].command_result.get_output(),
                OutputMode::Stdout => self.results_stdout[&result_index].command_result.get_stdout(),
                OutputMode::Stderr => self.results_stderr[&result_index].command_result.get_stderr(),
            };

            match self.is_regex_filter {
                true => {
                    let re = Regex::new(&self.filtered_text).unwrap();
                    let regex_match = re.is_match(&result_text);
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

        eprintln!("update_result: push history"); // Debug
        let mut selected = self.history_area.get_state_select();
        if is_push {
            match self.output_mode {
                OutputMode::Output => {
                    self.add_history(result_index, selected)
                },
                OutputMode::Stdout => {
                    if is_update_stdout {
                        self.add_history(result_index, selected)
                    }
                },
                OutputMode::Stderr => {
                    if is_update_stderr {
                        self.add_history(result_index, selected)
                    }
                },
            }
        }
        selected = self.history_area.get_state_select();

        // update hisotry area
        eprintln!("update_result: update hisotry area"); // Debug
        if is_limit_over {
            self.reset_history(selected);
        }

        if is_running_app{
            // update WatchArea
            self.set_output_data(selected);
        }

        true
    }

    /// Insert CommandResult into the results of each output mode.
    /// The return value is `result_index` and a bool indicating whether stdout/stderr has changed.
    /// Returns true if there is a change in stdout/stderr.
    fn insert_result(&mut self, result: CommandResult) -> (usize, bool, bool, bool) {
        eprintln!("insert_result: start"); // Debug
        let rc_result = Rc::new(result);
        let mut rc_output_result = ResultItems {
            command_result: Rc::clone(&rc_result),
            summary: HistorySummary::init(),
        };

        eprintln!("insert_result: get result index"); // Debug
        let result_index = self.results.keys().max().unwrap_or(&0) + 1;
        if result_index > 0 {
            let latest_num = result_index - 1;
            let latest_result = self.results[&latest_num].clone();
            eprintln!("insert_result: get result index: before summary.clac"); // Debug
            rc_output_result.summary.calc(&latest_result.command_result.get_output(), &rc_output_result.command_result.get_output(), self.enable_summary_char);
            eprintln!("insert_result: get result index: after summary.clac"); // Debug
        }
        self.results.insert(result_index, rc_output_result.clone());

        eprintln!("insert_result: create result stdout"); // Debug
        // create result_stdout
        let stdout_latest_index = get_results_latest_index(&self.results_stdout);
        let before_result_stdout = &self.results_stdout[&stdout_latest_index].command_result.get_stdout();
        let result_stdout = &rc_result.get_stdout();

        eprintln!("insert_result: create result stderr"); // Debug
        // create result_stderr
        let stderr_latest_index = get_results_latest_index(&self.results_stderr);
        let before_result_stderr = &self.results_stderr[&stderr_latest_index].command_result.get_stderr();
        let result_stderr = &rc_result.get_stderr();

        eprintln!("insert_result: append results_stdout"); // Debug
        // append results_stdout
        let mut is_stdout_update = false;
        if before_result_stdout != result_stdout {
            is_stdout_update = true;
            let mut rc_stdout_result = ResultItems {
                command_result: Rc::clone(&rc_result),
                summary: HistorySummary::init(),
            };
            rc_stdout_result.summary.calc(before_result_stdout, result_stdout, self.enable_summary_char);
            self.results_stdout.insert(result_index, rc_stdout_result);
        }

        eprintln!("insert_result: append results_stderr"); // Debug
        // append results_stderr
        let mut is_stderr_update = false;
        if before_result_stderr != result_stderr {
            is_stderr_update = true;
            let mut rc_stderr_result = ResultItems {
                command_result: Rc::clone(&rc_result),
                summary: HistorySummary::init(),
            };
            rc_stderr_result.summary.calc(before_result_stderr, result_stderr, self.enable_summary_char);
            self.results_stderr.insert(result_index, rc_stderr_result);
        }

        eprintln!("insert_result: limit check"); // Debug
        // limit check
        let mut is_limit_over = false;
        if self.limit > 0 {
            let limit = self.limit as usize;
            if self.results.len() > limit {
                let mut keys: Vec<_> = self.results.keys().cloned().collect();
                keys.sort();

                let remove_count = self.results.len() - limit;

                for key in keys.iter().take(remove_count) {
                    self.results.remove(key);
                }

                is_limit_over = true;
            }

            if self.results_stdout.len() > limit {
                let mut keys: Vec<_> = self.results_stdout.keys().cloned().collect();
                keys.sort();

                let remove_count = self.results_stdout.len() - limit;

                for key in keys.iter().take(remove_count) {
                    self.results_stdout.remove(key);
                }

                is_limit_over = true;
            }

            if self.results_stderr.len() > limit {
                let mut keys: Vec<_> = self.results_stderr.keys().cloned().collect();
                keys.sort();

                let remove_count = self.results_stderr.len() - limit;

                for key in keys.iter().take(remove_count) {
                    self.results_stderr.remove(key);
                }

                is_limit_over = true;
            }
        }

        return (result_index, is_limit_over,  is_stdout_update, is_stderr_update);
    }

    ///
    fn get_normal_input_key(&mut self, terminal_event: crossterm::event::Event) {
        // if exit window
        match self.window {
            ActiveWindow::Exit => {
                // match key event
                match terminal_event {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('y') => {
                                    self.exit();
                                    return;
                                },
                                KeyCode::Char('q') => {
                                    self.exit();
                                    return;
                                },
                                KeyCode::Char('n') => {
                                    self.window = ActiveWindow::Normal;
                                    return;
                                },
                                KeyCode::Char('h') => {
                                    self.window = ActiveWindow::Help;
                                    return;
                                },
                                // default
                                _ => {}
                            }
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }

        if let Some(event_content) = self.get_input_action(&terminal_event) {
            let action = event_content.action;
            match self.window {
                ActiveWindow::Normal => {
                    match action {
                        InputAction::Up => self.action_up(), // Up
                        InputAction::WatchPaneUp => self.action_watch_up(), // Watch Pane Up
                        InputAction::HistoryPaneUp => self.action_history_up(), // History Pane Up
                        InputAction::Down => self.action_down(), // Dow
                        InputAction::WatchPaneDown => self.action_watch_down(), // Watch Pane Down
                        InputAction::HistoryPaneDown => self.action_history_down(), // History Pane Down
                        InputAction::PageUp => self.action_pgup(), // PageUp
                        InputAction::WatchPanePageUp => self.action_watch_pgup(), // Watch Pane PageUp
                        InputAction::HistoryPanePageUp => self.action_history_pgup(), // History Pane PageUp
                        InputAction::PageDown => self.action_pgdn(), // PageDown
                        InputAction::WatchPanePageDown => self.action_watch_pgdn(), // Watch Pane PageDown
                        InputAction::HistoryPanePageDown => self.action_history_pgdn(), // History Pane PageDown
                        InputAction::MoveTop => self.action_top(), // MoveTop
                        InputAction::WatchPaneMoveTop => self.watch_area.scroll_home(), // Watch Pane MoveTop
                        InputAction::HistoryPaneMoveTop => self.action_history_top(), // History Pane MoveTop
                        InputAction::MoveEnd => self.action_end(), // MoveEnd
                        InputAction::WatchPaneMoveEnd => self.watch_area.scroll_end(), // Watch Pane MoveEnd
                        InputAction::HistoryPaneMoveEnd => self.action_history_end(), // History Pane MoveEnd
                        InputAction::ToggleForcus => self.toggle_area(), // ToggleForcus
                        InputAction::ForcusWatchPane => self.select_watch_pane(), // ForcusWatchPane
                        InputAction::ForcusHistoryPane => self.select_history_pane(), // ForcusHistoryPane
                        InputAction::Quit => {
                            if self.disable_exit_dialog {
                                self.exit();
                            } else {
                                self.show_exit_popup();
                            }
                        }, // Quit
                        InputAction::Reset => self.action_normal_reset(), // Reset   TODO: method分離したらちゃんとResetとしての機能を実装
                        InputAction::Cancel => self.action_normal_reset(), // Cancel   TODO: method分離したらちゃんとResetとしての機能を実装
                        InputAction::ForceCancel => self.action_force_reset(),
                        InputAction::Help => self.toggle_window(), // Help
                        InputAction::ToggleColor => self.set_ansi_color(!self.ansi_color), // ToggleColor
                        InputAction::ToggleLineNumber => self.set_line_number(!self.line_number), // ToggleLineNumber
                        InputAction::ToggleReverse => self.set_reverse(!self.reverse), // ToggleReverse
                        InputAction::ToggleMouseSupport => self.set_mouse_events(!self.mouse_events), // ToggleMouseSupport
                        InputAction::ToggleViewPaneUI => self.show_ui(!self.show_header), // ToggleViewPaneUI
                        InputAction::ToggleViewHistoryPane => self.show_history(!self.show_history), // ToggleViewHistory
                        InputAction::ToggleBorder => self.set_border(!self.is_border), // ToggleBorder
                        InputAction::ToggleScrollBar => self.set_scroll_bar(!self.is_scroll_bar), // ToggleScrollBar
                        InputAction::ToggleBorderWithScrollBar => {
                            self.set_border(!self.is_border);
                            self.set_scroll_bar(!self.is_scroll_bar);
                        }, // ToggleBorderWithScrollBar
                        InputAction::ToggleDiffMode => self.toggle_diff_mode(), // ToggleDiffMode
                        InputAction::SetDiffModePlane => self.set_diff_mode(DiffMode::Disable), // SetDiffModePlane
                        InputAction::SetDiffModeWatch => self.set_diff_mode(DiffMode::Watch), // SetDiffModeWatch
                        InputAction::SetDiffModeLine => self.set_diff_mode(DiffMode::Line), // SetDiffModeLine
                        InputAction::SetDiffModeWord => self.set_diff_mode(DiffMode::Word), // SetDiffModeWord
                        InputAction::SetDiffOnly => self.set_is_only_diffline(!self.is_only_diffline), // SetOnlyDiffLine
                        InputAction::ToggleOutputMode => self.toggle_output(), // ToggleOutputMode
                        InputAction::SetOutputModeOutput => self.set_output_mode(OutputMode::Output), // SetOutputModeOutput
                        InputAction::SetOutputModeStdout => self.set_output_mode(OutputMode::Stdout), // SetOutputModeStdout
                        InputAction::SetOutputModeStderr => self.set_output_mode(OutputMode::Stderr), // SetOutputModeStderr
                        InputAction::NextKeyword => self.action_next_keyword(), // NextKeyword
                        InputAction::PrevKeyword => self.action_previous_keyword(), // PreviousKeyword
                        InputAction::ToggleHistorySummary => self.set_history_summary(!self.is_history_summary), // ToggleHistorySummary
                        InputAction::IntervalPlus => self.increase_interval(), // IntervalPlus
                        InputAction::IntervalMinus => self.decrease_interval(), // IntervalMinus
                        InputAction::ChangeFilterMode => self.set_input_mode(InputMode::Filter), // Change Filter Mode(plane text).
                        InputAction::ChangeRegexFilterMode => self.set_input_mode(InputMode::RegexFilter), // Change Filter Mode(regex text).

                        // MouseScrollDown
                        InputAction::MouseScrollDown => {
                            if let Event::Mouse(mouse) = terminal_event {
                                self.mouse_scroll_down(mouse.column, mouse.row)
                            } else {
                                self.mouse_scroll_down(0, 0)
                            }
                        },

                        // MouseScrollUp
                        InputAction::MouseScrollUp => {
                            if let Event::Mouse(mouse) = terminal_event {
                                self.mouse_scroll_up(mouse.column, mouse.row)
                            } else {
                                self.mouse_scroll_up(0, 0)
                            }
                        },

                        // MouseButtonLeft
                        InputAction::MouseButtonLeft => {
                            if let Event::Mouse(mouse) = terminal_event {
                                self.mouse_click_left(mouse.column, mouse.row)
                            } else {
                                self.mouse_click_left(0, 0)
                            }
                        },

                        // default
                        _ => {}
                    }
                }
                ActiveWindow::Help => {
                    match action {
                        // Common input key
                        InputAction::Up => self.action_up(), // Up
                        InputAction::Down => self.action_down(), // Down
                        InputAction::PageUp => self.action_pgup(), // PageUp
                        InputAction::PageDown => self.action_pgdn(), // PageDown
                        InputAction::MoveTop => self.action_top(), // MoveTop
                        InputAction::MoveEnd => self.action_end(), // MoveEnd
                        InputAction::Help => self.toggle_window(), // Help
                        InputAction::Quit => {
                            if self.disable_exit_dialog {
                                self.exit();
                            } else {
                                self.show_exit_popup();
                            }
                        },
                        InputAction::Cancel => self.toggle_window(), // Cancel (Close help window with Cancel.)

                        // MouseScrollDown
                        InputAction::MouseScrollDown => {
                            if let Event::Mouse(mouse) = terminal_event {
                                self.mouse_scroll_down(mouse.column, mouse.row)
                            } else {
                                self.mouse_scroll_down(0, 0)
                            }
                        },

                        // MouseScrollUp
                        InputAction::MouseScrollUp => {
                            if let Event::Mouse(mouse) = terminal_event {
                                self.mouse_scroll_up(mouse.column, mouse.row)
                            } else {
                                self.mouse_scroll_up(0, 0)
                            }
                        },

                        // default
                        _ => {}
                    }
                },
                ActiveWindow::Exit => {
                    match action {
                        InputAction::Quit => self.exit(), // Quit
                        InputAction::Cancel => self.exit(), // Cancel
                        InputAction::Reset => self.window = ActiveWindow::Normal, // Reset
                        _ => {}
                    }
                }
            }

            return
        }
    }

    ///
    fn get_filter_input_key(&mut self, is_regex: bool, terminal_event: crossterm::event::Event) {
        if let Some(event_content) = self.keymap.get(&terminal_event) {
            let action = event_content.action;
            match action {
                InputAction::Cancel => self.action_input_reset(),
                _ => self.get_default_filter_input_key(is_regex, terminal_event),
            }
        } else {
            self.get_default_filter_input_key(is_regex, terminal_event)
        }
    }

    ///
    fn get_default_filter_input_key(&mut self, is_regex: bool, terminal_event: crossterm::event::Event) {
        if let Event::Key(key) = terminal_event {
            if key.kind == KeyEventKind::Press {
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

                        let selected = self.history_area.get_state_select();
                        self.reset_history(selected);

                        // update WatchArea
                        self.watch_area.set_keyword(self.filtered_text.clone(), is_regex);
                        self.set_output_data(selected);
                    }

                    // default
                    _ => {}
                }
            }
        }
    }

    ///
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
            _ => {},
        }
    }

    ///
    fn show_exit_popup(&mut self) {
        self.window = ActiveWindow::Exit;
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
    fn add_history(&mut self, result_index: usize, selected: usize) {
        // Switch the result depending on the output mode.
        let results = match self.output_mode {
            OutputMode::Output => &self.results,
            OutputMode::Stdout => &self.results_stdout,
            OutputMode::Stderr => &self.results_stderr,
        };

        // update history
        let timestamp = &results[&result_index].command_result.timestamp;
        let status = &results[&result_index].command_result.status;

        let history_summary = results[&result_index].summary.clone();

        self.history_area.update(
            timestamp.to_string(),
            *status,
            result_index as u16,
            history_summary
        );

        // update selected
        if selected != 0 {
            self.history_area.previous(1);
        }
    }

    ///
    pub fn show_ui(&mut self, visible: bool) {
        self.show_header = visible;
        self.show_history = visible;

        self.history_area.set_hide_header(!visible);
        self.watch_area.set_hide_header(!visible);

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
    fn action_normal_reset(&mut self) {
        if self.is_filtered {
            // unset filter
            self.is_filtered = false;
            self.is_regex_filter = false;
            self.filtered_text = "".to_string();
            self.header_area.input_text = self.filtered_text.clone();
            self.set_input_mode(InputMode::None);

            let selected = self.history_area.get_state_select();
            self.reset_history(selected);

            // update WatchArea
            self.watch_area.reset_keyword();
            self.set_output_data(selected);
        } else if 0 != self.history_area.get_state_select() {
            // set latest history
            self.reset_history(0);
            self.set_output_data(0);
        } else {
            // exit popup
            self.show_exit_popup()
        }
    }

    ///
    fn action_force_reset(&mut self) {
        if self.is_filtered {
            // unset filter
            self.is_filtered = false;
            self.is_regex_filter = false;
            self.filtered_text = "".to_string();
            self.header_area.input_text = self.filtered_text.clone();
            self.set_input_mode(InputMode::None);

            let selected = self.history_area.get_state_select();
            self.reset_history(selected);

            // update WatchArea
            self.watch_area.reset_keyword();
            self.set_output_data(selected);
        } else if 0 != self.history_area.get_state_select() {
            // set latest history
            self.reset_history(0);
            self.set_output_data(0);
        } else {
            self.exit();
        }
    }


    ///
    fn action_up(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {
                    self.action_watch_up()
                }
                ActiveArea::History => {
                    self.action_history_up()
                }
            },
            ActiveWindow::Help => {
                self.help_window.scroll_up(1);
            }
            _ => {},
        }
    }

    ///
    fn action_watch_up(&mut self) {
        // scroll up watch
        self.watch_area.scroll_up(1);
    }

    ///
    fn action_history_up(&mut self) {
        // move next history
        self.history_area.next(1);

        // get now selected history
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    fn action_down(&mut self) {
        match self.window {
            ActiveWindow::Normal => match self.area {
                ActiveArea::Watch => {
                    self.action_watch_down()
                }
                ActiveArea::History => {
                    self.action_history_down()
                }
            },
            ActiveWindow::Help => {
                self.help_window.scroll_down(1);
            }
            _ => {},
        }
    }

    ///
    fn action_watch_down(&mut self) {
        // scroll up watch
        self.watch_area.scroll_down(1);
    }

    ///
    fn action_history_down(&mut self) {
        // move previous history
        self.history_area.previous(1);

        // get now selected history
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    fn action_pgup(&mut self) {
        match self.window {
            ActiveWindow::Normal =>
                match self.area {
                    ActiveArea::Watch => {
                        self.action_watch_pgup();
                    },
                    ActiveArea::History => {
                        self.action_history_pgup();
                    }
                },
            ActiveWindow::Help => {
                self.help_window.page_up();
            }
            _ => {},
        }
    }

    ///
    fn action_watch_pgup(&mut self) {
        let mut page_height = self.watch_area.get_area_size();
        if page_height > 1 {
            page_height = page_height - 1
        }

        // scroll up watch
        self.watch_area.scroll_up(page_height);
    }

    ///
    fn action_history_pgup(&mut self) {
        // move next history
        let area_size = self.history_area.area.height;
        let move_size = if area_size > 1 {
            area_size - 1
        } else {
            1
        };

        // up
        self.history_area.next(move_size as usize);

        // get now selected history
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    fn action_pgdn(&mut self) {
        match self.window {
            ActiveWindow::Normal =>
                match self.area {
                    ActiveArea::Watch => {
                        self.action_watch_pgdn();
                    },
                    ActiveArea::History => {

                        self.action_history_pgdn();
                    },
                },
            ActiveWindow::Help => {
                self.help_window.page_down();
            }
            _ => {},
        }
    }

    ///
    fn action_watch_pgdn(&mut self) {
        let mut page_height = self.watch_area.get_area_size();
        if page_height > 1 {
            page_height = page_height - 1
        }

        // scroll up watch
        self.watch_area.scroll_down(page_height);
    }

    ///
    fn action_history_pgdn(&mut self) {
        // move previous history
        let area_size = self.history_area.area.height;
        let move_size = if area_size > 1 {
            area_size - 1
        } else {
            1
        };

        // down
        self.history_area.previous(move_size as usize);

        // get now selected history
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    fn action_top(&mut self) {
        match self.window {
            ActiveWindow::Normal =>
                match self.area {
                ActiveArea::Watch => self.watch_area.scroll_home(),
                ActiveArea::History => self.action_history_top(),
            },
            ActiveWindow::Help => {
                self.help_window.scroll_top();
            }
            _ => {},
        }
    }

    ///
    fn action_history_top(&mut self) {
        // move latest history move size
        let hisotory_size = self.history_area.get_history_size();
        self.history_area.next(hisotory_size);

        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    fn action_end(&mut self) {
        match self.window {
            ActiveWindow::Normal =>
                match self.area {
                    ActiveArea::Watch => self.watch_area.scroll_end(),
                    ActiveArea::History => self.action_history_end(),
                },
            ActiveWindow::Help => {
                self.help_window.scroll_end();
            }
            _ => {},
        }
    }

    ///
    fn action_history_end(&mut self) {
        // get end history move size
        let hisotory_size = self.history_area.get_history_size();
        let move_size = if hisotory_size > 1 {
            hisotory_size - 1
        } else {
            1
        };

        // move end
        self.history_area.previous(move_size);

        // get now selected history
        let selected = self.history_area.get_state_select();
        self.set_output_data(selected);
    }

    ///
    fn action_input_reset(&mut self) {
        self.header_area.input_text = self.filtered_text.clone();
        self.set_input_mode(InputMode::None);

        let selected = self.history_area.get_state_select();
        self.reset_history(selected);

        // update WatchArea
        self.set_output_data(selected);
    }

    ///
    fn action_previous_keyword(&mut self) {
        self.watch_area.previous_keyword();
    }

    ///
    fn action_next_keyword(&mut self) {
        self.watch_area.next_keyword();
    }

    ///
    fn select_watch_pane(&mut self) {
        if let ActiveWindow::Normal = self.window {
            self.area = ActiveArea::Watch;

            // set active window to header.
            self.header_area.set_active_area(self.area);
            self.header_area.update();
        }
    }

    ///
    fn select_history_pane(&mut self) {
        if let ActiveWindow::Normal = self.window {
            self.area = ActiveArea::History;

            // set active window to header.
            self.header_area.set_active_area(self.area);
            self.header_area.update();
        }
    }

    ///
    fn mouse_click_left(&mut self, column: u16, row: u16) {
        // check in hisotry area
        let is_history_area = check_in_area(self.history_area.area, column, row);
        if is_history_area {
            let headline_count = self.history_area.area.y;
            self.history_area.click_row(row - headline_count);

            let selected = self.history_area.get_state_select();
            self.set_output_data(selected);
        }
    }

    /// Mouse wheel always scroll up 2 lines.
    fn mouse_scroll_up(&mut self, column: u16, row: u16) {
        match self.window {
            ActiveWindow::Normal => {
                if column == 0 && row == 0 {
                    match self.area {
                        ActiveArea::Watch => {
                            self.watch_area.scroll_up(2);
                        },
                        ActiveArea::History => {
                            self.history_area.next(1);

                            let selected = self.history_area.get_state_select();
                            self.set_output_data(selected);
                        }
                    }
                } else {
                    let is_history_area = check_in_area(self.history_area.area, column, row);
                    if is_history_area {
                        self.history_area.next(1);

                        let selected = self.history_area.get_state_select();
                        self.set_output_data(selected);
                    } else {
                        self.watch_area.scroll_up(2);
                    }
                }

            },
            ActiveWindow::Help => {
                self.help_window.scroll_up(2);
            },
            _ => {},
        }
    }

    /// Mouse wheel always scroll down 2 lines.
    fn mouse_scroll_down(&mut self, column: u16, row: u16) {
        match self.window {
            ActiveWindow::Normal => {
                if column == 0 && row == 0 {
                    match self.area {
                        ActiveArea::Watch => {
                            self.watch_area.scroll_down(2);
                        },
                        ActiveArea::History => {
                            self.history_area.previous(1);

                            let selected = self.history_area.get_state_select();
                            self.set_output_data(selected);
                        }
                    }
                } else {
                    let is_history_area = check_in_area(self.history_area.area, column, row);
                    if is_history_area {
                        self.history_area.previous(1);

                        let selected = self.history_area.get_state_select();
                        self.set_output_data(selected);
                    } else {
                        self.watch_area.scroll_down(2);
                    }
                }
            },
            ActiveWindow::Help => {
                self.help_window.scroll_down(2);
            },
            _ => {},
        }
    }

    ///
    fn exit(&mut self) {
        self.tx.send(AppEvent::Exit)
            .expect("send error hwatch exit.");
    }
}

/// Checks whether the area where the mouse cursor is currently located is within the specified area.
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
    result
}

fn get_near_index(results: &HashMap<usize, ResultItems>, index: usize) -> usize {
    let keys = results.keys().cloned().collect::<Vec<usize>>();

    if keys.contains(&index) {
        return index;
    } else if index == 0 {
        return index;
    } else {
        let min = keys.iter().min().unwrap();
        if *min >= index {
            // return get_results_previous_index(results, index);
            return get_results_next_index(results, index);
        }
        // return get_results_next_index(results, index)
        return get_results_previous_index(results, index)
    }
}

fn get_results_latest_index(results: &HashMap<usize, ResultItems>) -> usize {
    let keys = results.keys().cloned().collect::<Vec<usize>>();

    // return keys.iter().max().unwrap();
    let max: usize = match keys.iter().max() {
        Some(n) => *n,
        None => 0,
    };

    return max;
}

fn get_results_previous_index(results: &HashMap<usize, ResultItems>, index: usize) -> usize {
    // get keys
    let mut keys: Vec<_> = results.keys().cloned().collect();
    keys.sort();

    let mut previous_index: usize = 0;
    for &k in &keys {
        if index == k {
            break;
        }
        previous_index = k;
    }

    return previous_index;

}

fn get_results_next_index(results: &HashMap<usize, ResultItems>, index: usize) -> usize {
    // get keys
    let mut keys: Vec<_> = results.keys().cloned().collect();
    keys.sort();

    let mut next_index: usize = 0;
    for &k in &keys {
        if index < k {
            next_index = k;
            break;
        }
    }

    return next_index;
}
