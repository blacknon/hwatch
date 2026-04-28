// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// TODO: historyの一個前、をdiffで取れるようにする(今は問答無用でVecの1個前のデータを取得しているから、ちょっと違う方法を取る?)
// TODO: log load時の追加処理がなんか変(たぶん、log load時に処理したresultをログに記録しちゃってる？？？)
//       →多分直った？と思うけど、要テスト

#[path = "app_actions.rs"]
mod actions;
#[path = "app_input.rs"]
mod input;
#[path = "app_render.rs"]
mod render;
#[path = "app_results.rs"]
mod results;

use self::results::get_near_index;
#[cfg(test)]
use self::results::{command_results_equivalent, gen_diff_only_data, gen_result_items};

// module
use crossbeam_channel::{Receiver, Sender};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    io::{self, Write},
    time::Duration,
};
use tui::{backend::Backend, style::Color, Terminal};

// local module
use crate::common::OutputMode;
use crate::event::AppEvent;
use crate::exec::CommandResult;
use crate::header::HeaderArea;
use crate::help::HelpWindow;
use crate::history::{HistoryArea, HistorySummary};
use crate::hwatch_ansi::get_ansi_strip_str;
use crate::hwatch_diffmode::DiffMode;
use crate::keymap::{default_keymap, Keymap};
use crate::output;
use crate::watch::WatchArea;
// local const
use crate::SharedInterval;
use crate::DEFAULT_TAB_SIZE;

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
    Delete,
    Clear,
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
pub struct ResultItems {
    pub command_result: CommandResult,
    pub summary: HistorySummary,

    // Strcut elements to with keyword filter.
    // ResultItems are created for each Output Type, so only one is generated in Sturct.
    pub diff_only_data: Vec<u8>,
}

impl ResultItems {
    /// Create a new ResultItems.
    pub fn default() -> Self {
        Self {
            command_result: CommandResult::default(),
            summary: HistorySummary::init(),

            diff_only_data: vec![],
        }
    }

    pub fn get_diff_only_data(&self, is_color: bool) -> String {
        if is_color {
            get_ansi_strip_str(&String::from_utf8_lossy(&self.diff_only_data))
        } else {
            String::from_utf8_lossy(&self.diff_only_data).to_string()
        }
    }
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
    after_command_result_write_file: bool,

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
    exit_on_change: Option<u32>,

    ///
    exit_on_change_armed: bool,

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

    //
    diff_mode: usize,

    ///
    diff_modes: Vec<Arc<Mutex<Box<dyn DiffMode>>>>,

    ///
    is_only_diffline: bool,

    ///
    ignore_spaceblock: bool,

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
    interval: SharedInterval,

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
impl App<'_> {
    ///
    pub fn new(
        tx: Sender<AppEvent>,
        rx: Receiver<AppEvent>,
        interval: SharedInterval,
        diff_modes: Vec<Arc<Mutex<Box<dyn DiffMode>>>>,
        diff_mode_width: usize,
    ) -> Self {
        // Create Default DiffMode
        let diff_mode_counter = 0;
        let mutex_diff_mode = Arc::clone(&diff_modes[diff_mode_counter]);

        // method at create new view trail.
        Self {
            keymap: default_keymap(),

            area: ActiveArea::History,
            window: ActiveWindow::Normal,

            limit: 0,

            after_command: "".to_string(),
            after_command_result_write_file: false,
            ansi_color: false,
            line_number: false,
            reverse: false,
            disable_exit_dialog: false,
            show_history: true,
            show_header: true,

            is_beep: false,
            exit_on_change: None,
            exit_on_change_armed: false,
            is_border: false,
            is_history_summary: false,
            is_scroll_bar: false,
            is_filtered: false,
            is_regex_filter: false,
            filtered_text: "".to_string(),

            input_mode: InputMode::None,
            output_mode: OutputMode::Output,
            diff_mode: diff_mode_counter,
            diff_modes: diff_modes,
            is_only_diffline: false,
            ignore_spaceblock: false,

            results: HashMap::new(),
            results_stdout: HashMap::new(),
            results_stderr: HashMap::new(),

            enable_summary_char: false,

            interval: interval.clone(),
            tab_size: DEFAULT_TAB_SIZE,

            header_area: {
                let mut header_area = HeaderArea::new(interval.clone(), mutex_diff_mode.clone());
                header_area.set_diff_mode_width(diff_mode_width);
                header_area
            },
            history_area: HistoryArea::new(),
            watch_area: WatchArea::new(),

            help_window: HelpWindow::new(default_keymap()),

            mouse_events: false,

            printer: output::Printer::new(mutex_diff_mode.clone()),

            done: false,
            logfile: "".to_string(),
            tx,
            rx,
        }
    }

    ///
    pub fn run<B: Backend + Write>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        // set history setting
        self.history_area
            .set_enable_char_diff(self.enable_summary_char);

        if !self.results.is_empty() {
            let selected = self.history_area.get_state_select();
            let new_selected = self.reset_history(selected);
            self.set_output_data(new_selected);
        } else {
            self.history_area.next(1);
        }

        let mut update_draw = true;

        self.printer
            .set_batch(false)
            .set_color(self.ansi_color)
            .set_diff_mode(self.diff_modes[self.diff_mode].clone())
            .set_line_number(self.line_number)
            .set_output_mode(self.output_mode)
            .set_tab_size(self.tab_size)
            .set_only_diffline(self.is_only_diffline)
            .set_ignore_spaceblock(self.ignore_spaceblock);

        loop {
            if matches!(self.exit_on_change, Some(0)) {
                self.done = true;
            }
            if self.done {
                return Ok(());
            }

            // Draw data
            if update_draw {
                match terminal.draw(|f| self.draw(f)) {
                    Ok(_) => update_draw = false,
                    Err(err) if is_retryable_terminal_error(&err.to_string()) => {}
                    Err(err) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("{err}"),
                        ))
                    }
                }
            }

            // get event
            match self.rx.recv_timeout(Duration::from_millis(100)) {
                Ok(AppEvent::Redraw) => update_draw = true,

                // Get terminal event.
                Ok(AppEvent::TerminalEvent(terminal_event)) => {
                    self.get_event(terminal_event);
                    update_draw = true;
                }

                // Get command result.
                Ok(AppEvent::OutputUpdate(exec_result)) => {
                    let changed = self.create_result_items(exec_result, true);

                    if changed && self.is_beep {
                        println!("\x07")
                    }

                    self.handle_exit_on_change(changed);
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
    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.keymap = keymap.clone();
        self.help_window = HelpWindow::new(self.keymap.clone());
    }

    ///
    pub fn set_after_command(&mut self, command: String) {
        self.after_command = command;
    }

    ///
    pub fn set_after_command_result_write_file(&mut self, write_file: bool) {
        self.after_command_result_write_file = write_file;
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
        if !self.results.is_empty() {
            // Switch the result depending on the output mode.
            let results = match self.output_mode {
                OutputMode::Output => &self.results,
                OutputMode::Stdout => &self.results_stdout,
                OutputMode::Stderr => &self.results_stderr,
            };
            let selected: usize = self.history_area.get_state_select();
            let new_selected = get_near_index(results, selected);
            let reseted_select = self.reset_history(new_selected);
            self.set_output_data(reseted_select);
        }
    }

    ///
    pub fn set_ansi_color(&mut self, ansi_color: bool) {
        self.ansi_color = ansi_color;

        self.header_area.set_ansi_color(ansi_color);
        self.header_area.update();

        self.printer.set_color(ansi_color);

        self.refresh_selected_watch_output();
    }

    ///
    pub fn set_beep(&mut self, beep: bool) {
        self.is_beep = beep;
    }

    ///
    pub fn set_exit_on_change(&mut self, exit_on_change: Option<u32>) {
        self.exit_on_change = exit_on_change;
        self.exit_on_change_armed = false;
    }

    ///
    pub fn set_border(&mut self, border: bool) {
        self.is_border = border;

        // set border
        self.history_area.set_border(border);
        self.watch_area.set_border(border);

        self.refresh_selected_watch_output();
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

        self.refresh_selected_watch_output();
    }

    ///
    pub fn set_scroll_bar(&mut self, scroll_bar: bool) {
        self.is_scroll_bar = scroll_bar;

        // set scroll_bar
        self.history_area.set_scroll_bar(scroll_bar);
        self.watch_area.set_scroll_bar(scroll_bar);

        self.refresh_selected_watch_output();
    }

    ///
    pub fn set_watch_diff_colors(&mut self, fg: Option<Color>, bg: Option<Color>) {
        self.printer.set_watch_diff_colors(fg, bg);

        self.refresh_selected_watch_output();
    }

    ///
    pub fn set_line_number(&mut self, line_number: bool) {
        self.line_number = line_number;

        self.header_area.set_line_number(line_number);
        self.header_area.update();

        self.printer.set_line_number(line_number);

        self.refresh_selected_watch_output();
    }

    ///
    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;

        self.header_area.set_reverse(reverse);
        self.header_area.update();

        self.printer.set_reverse(reverse);

        self.refresh_selected_watch_output();
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
    pub fn set_mouse_events(&mut self, mouse_events: bool) {
        self.mouse_events = mouse_events;
        self.tx.send(AppEvent::ChangeFlagMouseEvent).unwrap();
    }

    ///
    pub fn set_wrap_mode(&mut self, wrap: bool) {
        self.watch_area.set_wrap_mode(wrap);

        self.refresh_selected_watch_output();
    }

    ///
    pub fn add_results(&mut self, results: Vec<CommandResult>) {
        for result in results {
            self.create_result_items(result, false);
        }
    }

    ///
    fn increase_interval(&mut self) {
        self.interval.write().unwrap().increase(0.5);
        self.header_area.update();
    }

    ///
    fn decrease_interval(&mut self) {
        self.interval.write().unwrap().decrease(0.5);
        self.header_area.update();
    }

    ///
    fn toggle_pause(&mut self) {
        self.interval.write().unwrap().toggle_pause();
        self.header_area.update();
    }

    ///
    pub fn set_diff_mode(&mut self, diff_mode: usize) {
        self.diff_mode = diff_mode;

        self.header_area
            .set_diff_mode(self.diff_modes[self.diff_mode].clone());
        self.header_area.update();

        self.printer
            .set_diff_mode(self.diff_modes[self.diff_mode].clone());

        let selected = self.history_area.get_state_select();

        if !self.results.is_empty() {
            let reseted_select = self.reset_history(selected);
            self.set_output_data(reseted_select);
        } else {
            self.set_output_data(selected);
        }
    }

    ///
    pub fn set_is_only_diffline(&mut self, is_only_diffline: bool) {
        self.is_only_diffline = is_only_diffline;

        self.printer.set_only_diffline(is_only_diffline);

        self.header_area.set_is_only_diffline(is_only_diffline);
        self.header_area.update();

        let selected = self.history_area.get_state_select();
        if !self.results.is_empty() {
            let reseted_select = self.reset_history(selected);
            self.set_output_data(reseted_select);
        } else {
            self.set_output_data(selected);
        }
    }

    pub fn set_ignore_spaceblock(&mut self, ignore_spaceblock: bool) {
        self.ignore_spaceblock = ignore_spaceblock;
        self.printer.set_ignore_spaceblock(ignore_spaceblock);

        let selected = self.history_area.get_state_select();
        if !self.results.is_empty() {
            let reseted_select = self.reset_history(selected);
            self.set_output_data(reseted_select);
        } else {
            self.set_output_data(selected);
        }
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
}

fn is_retryable_terminal_error(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();

    normalized.contains("wouldblock")
        || normalized.contains("would block")
        || normalized.contains("interrupted")
        || normalized.contains("resource temporarily unavailable")
        || normalized.contains("temporarily unavailable")
        || normalized.contains("operation interrupted")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use std::io::{self, Write};
    use std::sync::RwLock;
    use tui::{
        backend::{Backend, ClearType, TestBackend, WindowSize},
        buffer::Cell,
        layout::{Position, Size},
    };

    use crate::diffmode_plane::DiffModeAtPlane;
    use crate::RunInterval;

    #[cfg(not(skip_proptest_tests))]
    use proptest::prelude::*;

    struct FlakyTestBackend {
        inner: TestBackend,
        fail_next_draw: bool,
    }

    impl FlakyTestBackend {
        fn new(width: u16, height: u16) -> Self {
            Self {
                inner: TestBackend::new(width, height),
                fail_next_draw: true,
            }
        }
    }

    impl Write for FlakyTestBackend {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Backend for FlakyTestBackend {
        fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a Cell)>,
        {
            if self.fail_next_draw {
                self.fail_next_draw = false;
                return Err(io::Error::from(io::ErrorKind::WouldBlock));
            }

            match self.inner.draw(content) {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn hide_cursor(&mut self) -> io::Result<()> {
            match self.inner.hide_cursor() {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn show_cursor(&mut self) -> io::Result<()> {
            match self.inner.show_cursor() {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn get_cursor_position(&mut self) -> io::Result<Position> {
            match self.inner.get_cursor_position() {
                Ok(position) => Ok(position),
                Err(err) => Err(err),
            }
        }

        fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
            match self.inner.set_cursor_position(position) {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn clear(&mut self) -> io::Result<()> {
            match self.inner.clear() {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
            match self.inner.clear_region(clear_type) {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn append_lines(&mut self, line_count: u16) -> io::Result<()> {
            match self.inner.append_lines(line_count) {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }

        fn size(&self) -> io::Result<Size> {
            match self.inner.size() {
                Ok(size) => Ok(size),
                Err(err) => Err(err),
            }
        }

        fn window_size(&mut self) -> io::Result<WindowSize> {
            match self.inner.window_size() {
                Ok(size) => Ok(size),
                Err(err) => Err(err),
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self.inner.flush() {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        }
    }

    fn test_diff_modes() -> Vec<Arc<Mutex<Box<dyn DiffMode>>>> {
        vec![Arc::new(Mutex::new(Box::new(DiffModeAtPlane::new())))]
    }

    #[test]
    fn retryable_terminal_errors_match_expected_messages() {
        assert!(is_retryable_terminal_error("WouldBlock"));
        assert!(is_retryable_terminal_error("operation would block"));
        assert!(is_retryable_terminal_error(
            "Resource temporarily unavailable (os error 35)"
        ));
        assert!(is_retryable_terminal_error("operation interrupted"));
        assert!(!is_retryable_terminal_error("permission denied"));
    }

    #[test]
    fn run_retries_retryable_draw_errors() {
        let (tx, rx) = unbounded();
        let interval = Arc::new(RwLock::new(RunInterval::default()));
        let mut app = App::new(tx.clone(), rx, interval, test_diff_modes(), 0);
        let mut terminal = Terminal::new(FlakyTestBackend::new(80, 24)).unwrap();

        tx.send(AppEvent::Exit).unwrap();

        let result = app.run(&mut terminal);
        assert!(result.is_ok(), "unexpected run() error: {result:?}");
    }

    #[test]
    fn invalid_regex_filter_input_does_not_enable_filtering() {
        let (tx, rx) = unbounded();
        let interval = Arc::new(RwLock::new(RunInterval::default()));
        let mut app = App::new(tx, rx, interval, test_diff_modes(), 0);

        app.set_input_mode(InputMode::RegexFilter);
        app.header_area.input_text = "[".to_string();

        app.get_default_filter_input_key(
            true,
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );

        assert!(app.input_mode == InputMode::RegexFilter);
        assert!(!app.is_filtered);
        assert!(!app.is_regex_filter);
        assert_eq!(app.filtered_text, "");
        assert_eq!(app.header_area.input_text, "[");
    }

    #[test]
    fn invalid_regex_filter_does_not_panic_during_match_checks() {
        let (tx, rx) = unbounded();
        let interval = Arc::new(RwLock::new(RunInterval::default()));
        let mut app = App::new(tx, rx, interval, test_diff_modes(), 0);

        app.is_regex_filter = true;
        app.filtered_text = "[".to_string();

        assert!(!app.matches_filter_text("sample output"));
    }

    #[test]
    fn gen_diff_only_data_collects_only_changed_lines() {
        let diff_only = gen_diff_only_data("same\nold\n", "same\nnew\n", false);

        assert_eq!(String::from_utf8(diff_only).unwrap(), "old\nnew\n");
    }

    #[test]
    fn gen_diff_only_data_returns_empty_when_inputs_match() {
        let diff_only = gen_diff_only_data("same\n", "same\n", false);

        assert!(diff_only.is_empty());
    }

    #[test]
    fn gen_diff_only_data_ignores_space_blocks_when_enabled() {
        let diff_only = gen_diff_only_data("alpha  beta\n", "alpha   beta\n", true);

        assert!(diff_only.is_empty());
    }

    #[test]
    fn gen_diff_only_data_preserves_change_order_across_lines() {
        let diff_only = gen_diff_only_data("a\nb\nc\n", "a\nx\nc\ny\n", false);

        assert_eq!(String::from_utf8(diff_only).unwrap(), "b\nx\ny\n");
    }

    #[test]
    fn gen_diff_only_data_detects_trailing_newline_changes() {
        let diff_only = gen_diff_only_data("alpha", "alpha\n", false);

        assert!(!diff_only.is_empty());
    }

    #[test]
    fn gen_diff_only_data_marks_each_inserted_line_when_starting_empty() {
        let diff_only = gen_diff_only_data("", "line1\nline2\n", false);

        assert_eq!(String::from_utf8(diff_only).unwrap(), "line1\nline2\n");
    }

    #[test]
    fn gen_result_items_keeps_stdout_and_stderr_diffs_separate() {
        let previous = CommandResult::default()
            .set_output(b"out-1\nerr-1\n".to_vec())
            .set_stdout(b"out-1\n".to_vec())
            .set_stderr(b"err-1\n".to_vec());
        let current = CommandResult::default()
            .set_output(b"out-2\nerr-2\n".to_vec())
            .set_stdout(b"out-2\n".to_vec())
            .set_stderr(b"err-2\n".to_vec());

        let (output_items, stdout_items, stderr_items) =
            gen_result_items(current, true, false, &previous, &previous, &previous);

        assert_eq!(
            String::from_utf8(output_items.diff_only_data).unwrap(),
            "out-1\nerr-1\nout-2\nerr-2\n"
        );
        assert_eq!(
            String::from_utf8(stdout_items.diff_only_data).unwrap(),
            "out-1\nout-2\n"
        );
        assert_eq!(
            String::from_utf8(stderr_items.diff_only_data).unwrap(),
            "err-1\nerr-2\n"
        );
        assert_eq!(
            (stdout_items.summary.line_add, stdout_items.summary.line_rem),
            (1, 1)
        );
        assert_eq!(
            (stderr_items.summary.line_add, stderr_items.summary.line_rem),
            (1, 1)
        );
    }

    #[test]
    fn gen_result_items_calculates_summary_per_output_stream() {
        let previous = CommandResult::default()
            .set_output(b"out-1\nerr-1\n".to_vec())
            .set_stdout(b"out-1\n".to_vec())
            .set_stderr(b"err-1\n".to_vec());
        let current = CommandResult::default()
            .set_output(b"out-2\nerr-1\n".to_vec())
            .set_stdout(b"out-2\n".to_vec())
            .set_stderr(b"err-1\n".to_vec());

        let (output_items, stdout_items, stderr_items) =
            gen_result_items(current, true, false, &previous, &previous, &previous);

        assert_eq!(
            (output_items.summary.line_add, output_items.summary.line_rem),
            (1, 1)
        );
        assert_eq!(
            (stdout_items.summary.line_add, stdout_items.summary.line_rem),
            (1, 1)
        );
        assert_eq!(
            (stderr_items.summary.line_add, stderr_items.summary.line_rem),
            (0, 0)
        );
        assert_eq!(
            (stderr_items.summary.char_add, stderr_items.summary.char_rem),
            (0, 0)
        );
    }

    #[test]
    fn command_results_equivalent_ignores_space_blocks_when_enabled() {
        let before = CommandResult::default()
            .set_output(b"alpha  beta\n".to_vec())
            .set_stdout(b"alpha  beta\n".to_vec())
            .set_stderr(b"".to_vec());
        let after = CommandResult::default()
            .set_output(b"alpha   beta\n".to_vec())
            .set_stdout(b"alpha   beta\n".to_vec())
            .set_stderr(b"".to_vec());

        assert!(command_results_equivalent(&before, &after, true));
        assert!(!command_results_equivalent(&before, &after, false));
    }

    #[cfg(not(skip_proptest_tests))]
    proptest! {
        #[test]
        fn gen_diff_only_data_is_empty_for_identical_inputs(text in "[^\0]{0,64}") {
            let diff_only = gen_diff_only_data(&text, &text, false);
            prop_assert!(diff_only.is_empty());
        }

        #[test]
        fn command_results_equivalent_is_reflexive(
            command in "[^\0]{0,64}",
            output in "[^\0]{0,64}",
            stdout in "[^\0]{0,64}",
            stderr in "[^\0]{0,64}",
            status in any::<bool>(),
        ) {
            let result = CommandResult {
                command,
                status,
                ..CommandResult::default()
            }
            .set_output(output.as_bytes().to_vec())
            .set_stdout(stdout.as_bytes().to_vec())
            .set_stderr(stderr.as_bytes().to_vec());

            prop_assert!(command_results_equivalent(&result, &result, false));
            prop_assert!(command_results_equivalent(&result, &result, true));
        }

        #[test]
        fn gen_diff_only_data_ignores_whitespace_only_changes_when_normalized(
            left in "[^\n\r]{0,32}",
            spaces_a in "[ \t]{1,8}",
            spaces_b in "[ \t]{1,8}",
            right in "[^\n\r]{0,32}",
        ) {
            let before = format!("{left}{spaces_a}{right}\n");
            let after = format!("{left}{spaces_b}{right}\n");

            prop_assume!(hwatch_diffmode::normalize_space_blocks(&before)
                == hwatch_diffmode::normalize_space_blocks(&after));

            let diff_only = gen_diff_only_data(&before, &after, true);
            prop_assert!(diff_only.is_empty());
        }
    }
}
