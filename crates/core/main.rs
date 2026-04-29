// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::empty_docs)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::extra_unused_lifetimes)]
#![allow(clippy::io_other_error)]
#![allow(clippy::items_after_test_module)]
#![allow(clippy::len_zero)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::useless_format)]
#![allow(clippy::wrong_self_convention)]

// v0.4.3
// TODO(blacknon): https://github.com/blacknon/hwatch/issues/42
// TODO(blacknon): diff modeをさらに複数用意し、選択・切り替えできるdiffをオプションから指定できるようにする(watchをold-watchにして、モダンなwatchをデフォルトにしたり)
// TODO(blacknon): formatを整える機能や、diff時に特定のフォーマットかどうかで扱いを変える機能について、追加する方法を模索する(プラグインか、もしくはパイプでうまいこときれいにする機能か？)
// TODO(blacknon): filter modeのハイライト表示の色を環境変数で定義できるようにする
// TODO(blacknon): filter modeの検索ヒット数を表示する(どうやってやろう…？というより、どこに表示させよう…？)
// TODO(blacknon): Windowsのバイナリをパッケージマネジメントシステムでインストール可能になるよう、Releaseでうまいこと処理をする
// TODO(blacknon): watchをモダンよりのものに変更する

// v0.5.0
// TODO(blacknon):
//   横に2画面表示するモードの追加
//     - diff modeの一種か？？
//     - 既存のモードとは違う種類のモードとして、カテゴリを分けて扱うべきかも？
//     - ウィンドウは2個にして、スクロールは連動させる必要がある
//     - 2画面のうち、左は前回の出力、右は今回の出力を表示するモードにする
//     - イメージ的にはdiff -yやvimのvertical diffみたいな感じである。

// v1.0.0
// TODO(blacknon): vimのように内部コマンドを利用した表示切り替え・出力結果の編集機能を追加する
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能を追加する(command mode option)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man
// TODO(blacknon): エラーなどのメッセージ表示領域の作成

// modules
extern crate hwatch_ansi;
extern crate hwatch_diffmode;
extern crate ratatui as tui;

use clap::error::ErrorKind;
use cli::{build_app, get_clap_matcher, should_continue_with_unreadable_logfile};
use common::load_logfile;
use crossbeam_channel::unbounded;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use diff_mode_registry::{calculate_diff_mode_header_width, register_diff_mode_name};
use hwatch_diffmode::DiffMode;
use interval::RunInterval;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

// local modules
mod app;
mod batch;
mod cli;
mod common;
mod completion;
mod diff_mode_registry;
mod diffmode_line;
mod diffmode_plane;
mod diffmode_watch;
mod errors;
mod event;
mod exec;
mod header;
mod help;
mod history;
mod interval;
mod keymap;
mod output;
mod plugin_diffmode;
mod popup;
mod view;
mod watch;

// const
pub const DEFAULT_TAB_SIZE: u16 = 4;
pub const HISTORY_WIDTH: u16 = 25;
pub const SHELL_COMMAND_EXECCMD: &str = "{COMMAND}";
pub const HISTORY_LIMIT: &str = "5000";
type SharedInterval = Arc<RwLock<RunInterval>>;

// const at Windows
#[cfg(windows)]
const SHELL_COMMAND: &str = "cmd /C";

// const at not Windows
#[cfg(not(windows))]
const SHELL_COMMAND: &str = "sh -c";

fn main() {
    match completion::handle_completion_request() {
        Ok(true) => {
            return;
        }
        Ok(false) => {}
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    }
    let mut cmd_app = build_app();
    let matcher = get_clap_matcher(cmd_app.clone());

    // Get options flag
    let batch = matcher.get_flag("batch");
    let compress = matcher.get_flag("compress");
    let precise = matcher.get_flag("precise");
    let exit_on_change = matcher.get_one::<u32>("chgexit").copied();

    // Get after command
    let after_command = matcher.get_one::<String>("after_command");
    let after_command_result_write_file = matcher.get_flag("after_command_result_write_file");

    // Get logfile
    let logfile = matcher.get_one::<String>("logfile");
    let force_logfile_overwrite = matcher.get_flag("force_logfile_overwrite");

    // check _logfile directory
    // TODO(blacknon): commonに移す？(ここで直書きする必要性はなさそう)
    let mut load_results = vec![];
    if let Some(logfile) = logfile {
        // logging log
        let log_path = Path::new(logfile);
        let log_dir = log_path.parent().unwrap_or_else(|| Path::new("."));
        let cur_dir = std::env::current_dir().expect("cannot get current directory");
        let abs_log_path = cur_dir.join(log_path);
        let abs_log_dir = cur_dir.join(log_dir);

        // check _log_dir exist
        if !abs_log_dir.exists() {
            let err = cmd_app.error(
                ErrorKind::ValueValidation,
                format!("directory {abs_log_dir:?} is not exists."),
            );
            err.exit();
        }

        // load logfile
        match load_logfile(abs_log_path.to_str().unwrap(), compress) {
            Ok(results) => {
                load_results = results;
            }
            Err(e) => {
                let mut is_overwrite_question = false;
                match e {
                    common::LoadLogfileError::LogfileEmpty => {
                        eprintln!("file {abs_log_path:?} is exists and empty.");
                        is_overwrite_question = true;
                    }
                    common::LoadLogfileError::LoadFileError(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => {}
                        _ => {
                            eprintln!("file {abs_log_path:?} is exists and load error.");
                            eprintln!("{err:?}");
                            is_overwrite_question = true;
                        }
                    },
                    common::LoadLogfileError::JsonParseError(err) => {
                        eprintln!("file {abs_log_path:?} is exists and json parse error.");
                        eprintln!("{err:?}");
                        is_overwrite_question = true;
                    }
                }

                if is_overwrite_question
                    && !should_continue_with_unreadable_logfile(
                        force_logfile_overwrite,
                        !batch && std::io::stdin().is_terminal(),
                    )
                {
                    std::process::exit(1);
                }
            }
        }
    }

    // Create channel
    let (tx, rx) = unbounded();

    // interval
    let shared_interval: SharedInterval = match matcher.get_one::<f64>("interval") {
        Some(override_interval) => SharedInterval::new(RunInterval::new(*override_interval).into()),
        None => SharedInterval::default(),
    };

    // history limit
    let default_limit: u32 = HISTORY_LIMIT.parse().unwrap();
    let limit = matcher.get_one::<u32>("limit").unwrap_or(&default_limit);

    // tab size
    let tab_size = *matcher
        .get_one::<u16>("tab_size")
        .unwrap_or(&DEFAULT_TAB_SIZE);

    // enable summary char
    let enable_summary_char = matcher.get_flag("enable_summary_char");

    // output mode
    let output_mode = match matcher.get_one::<String>("output").unwrap().as_str() {
        "output" => common::OutputMode::Output,
        "stdout" => common::OutputMode::Stdout,
        "stderr" => common::OutputMode::Stderr,
        _ => common::OutputMode::Output,
    };

    // Get Add keymap
    let keymap_options: Vec<&str> = matcher
        .get_many::<String>("keymap")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    // Parse Add Keymap
    let keymap = match keymap::generate_keymap(keymap_options) {
        Ok(keymap) => keymap,
        _ => {
            eprintln!("Failed to parse keymap.");
            std::process::exit(1);
        }
    };

    let watch_diff_fg = match std::env::var("HWATCH_WATCH_FG") {
        Ok(value) => match common::parse_ansi_color(&value) {
            Ok(color) => Some(color),
            Err(message) => {
                let err = cmd_app.error(
                    ErrorKind::ValueValidation,
                    format!("$HWATCH_WATCH_FG {message}"),
                );
                err.exit();
            }
        },
        Err(_) => None,
    };

    let watch_diff_bg = match std::env::var("HWATCH_WATCH_BG") {
        Ok(value) => match common::parse_ansi_color(&value) {
            Ok(color) => Some(color),
            Err(message) => {
                let err = cmd_app.error(
                    ErrorKind::ValueValidation,
                    format!("$HWATCH_WATCH_BG {message}"),
                );
                err.exit();
            }
        },
        Err(_) => None,
    };

    // set command
    let command_line: Vec<String>;
    if let Some(value) = matcher.get_many::<String>("command") {
        command_line = value.into_iter().cloned().collect();
    } else {
        // check load_results
        if load_results.is_empty() {
            let err = cmd_app.error(
                ErrorKind::InvalidValue,
                "command not specified and logfile is empty.".to_string(),
            );
            err.exit();
        }

        // set command
        let command = load_results.last().unwrap().command.clone();
        command_line = match shell_words::split(&command) {
            Ok(command_line) => command_line,
            Err(err) => {
                let err = cmd_app.error(
                    ErrorKind::ValueValidation,
                    format!("failed to restore command from logfile: {err}"),
                );
                err.exit();
            }
        };
    }

    // Start Command Thread
    {
        let m = matcher.clone();
        let tx = tx.clone();
        let shell_command = m.get_one::<String>("shell_command").unwrap().to_string();
        let command: Vec<_> = command_line;
        let is_exec = m.get_flag("exec");
        let is_pty = m.get_flag("use_pty");
        let run_interval_ptr = shared_interval.clone();
        let _ = thread::spawn(move || loop {
            let run_interval = run_interval_ptr.read().expect("Non poisoned block");
            let paused = run_interval.paused;
            let interval = run_interval.interval;
            drop(run_interval); // We manually drop here or else it locks anything else from reading/writing the interval
            let mut time_to_sleep: f64 = interval;

            if !paused {
                // Create cmd..
                let mut exe = exec::ExecuteCommand::new(tx.clone());

                // Set shell command
                exe.shell_command = shell_command.clone();

                // Set command
                exe.command = command.clone();

                // Set compress
                exe.is_compress = compress;

                // Set is exec flag.
                exe.is_exec = is_exec;
                exe.is_pty = is_pty;

                let before_start = SystemTime::now();
                // Exec command
                exe.exec_command();

                if precise {
                    let elapsed: f64 = SystemTime::now()
                        .duration_since(before_start)
                        .unwrap_or_default()
                        .as_secs_f64();
                    time_to_sleep = match elapsed > time_to_sleep {
                        true => 0_f64,
                        false => time_to_sleep - elapsed,
                    };
                }
            }

            std::thread::sleep(Duration::from_secs_f64(time_to_sleep));
        });
    }

    let mut diff_mode_name_to_index: HashMap<String, usize> = HashMap::new();

    // create diff_mode(plane)
    let diff_mode_plane = diffmode_plane::DiffModeAtPlane::new();

    // create diff_mode(watch)
    let diff_mode_watch = diffmode_watch::DiffModeAtWatch::new();

    // create diff_mode(line)
    let mut diff_mode_line = diffmode_line::DiffModeAtLineDiff::new();
    diff_mode_line.is_word_highlight = false;

    // create diff_mode(word)
    let mut diff_mode_word = diffmode_line::DiffModeAtLineDiff::new();
    diff_mode_word.is_word_highlight = true;

    // set diff_modes
    let mut diff_modes: Vec<Arc<Mutex<Box<dyn DiffMode>>>> = vec![
        Arc::new(Mutex::new(Box::new(diff_mode_plane))),
        Arc::new(Mutex::new(Box::new(diff_mode_watch))),
        Arc::new(Mutex::new(Box::new(diff_mode_line))),
        Arc::new(Mutex::new(Box::new(diff_mode_word))),
    ];

    for (index, name) in ["none", "watch", "line", "word"].into_iter().enumerate() {
        register_diff_mode_name(&mut diff_mode_name_to_index, name.to_string(), index).unwrap();
    }

    if let Some(plugin_paths) = matcher.get_many::<String>("diff_plugin") {
        for plugin_path in plugin_paths {
            let plugin_path = Path::new(plugin_path);
            let registration = match plugin_diffmode::load_plugin(plugin_path) {
                Ok(registration) => registration,
                Err(message) => {
                    let err = cmd_app.error(ErrorKind::Io, message);
                    err.exit();
                }
            };

            let plugin_name = registration.name;
            if let Err(message) =
                register_diff_mode_name(&mut diff_mode_name_to_index, plugin_name, diff_modes.len())
            {
                let err = cmd_app.error(ErrorKind::ArgumentConflict, message);
                err.exit();
            }

            diff_modes.push(Arc::new(Mutex::new(registration.mode)));
        }
    }

    // diff mode
    let diff_mode = if matcher.contains_id("differences") {
        let requested = matcher.get_one::<String>("differences").unwrap();
        match diff_mode_name_to_index.get(requested) {
            Some(index) => *index,
            None => {
                let available = diff_mode_name_to_index
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let err = cmd_app.error(
                    ErrorKind::InvalidValue,
                    format!("unknown diff mode '{requested}'. Available: {available}"),
                );
                err.exit();
            }
        }
    } else {
        0
    };

    let diff_mode_width = calculate_diff_mode_header_width(&diff_modes);

    // check batch mode
    if !batch {
        // On Windows, Ctrl+C can be delivered as a process signal before key input is read.
        // Convert it into the same key event as the default keymap (`ctrl-c=cancel`).
        let tx_ctrlc = tx.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = tx_ctrlc.send(event::AppEvent::TerminalEvent(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })));
        });

        // is watch mode
        // Create view
        let mut view = view::View::new(shared_interval.clone(), diff_modes)
            .set_tab_size(tab_size)
            .set_limit(*limit)
            .set_beep(matcher.get_flag("beep"))
            .set_exit_on_change(exit_on_change)
            .set_border(matcher.get_flag("border"))
            .set_scroll_bar(matcher.get_flag("with_scrollbar"))
            .set_mouse_events(matcher.get_flag("mouse"))
            // set keymap
            .set_keymap(keymap)
            // Set color in view
            .set_color(matcher.get_flag("color"))
            // Set watch diff highlight colors
            .set_watch_diff_colors(watch_diff_fg, watch_diff_bg)
            // Set line number in view
            .set_line_number(matcher.get_flag("line_number"))
            // Set reverse mode in view
            .set_reverse(matcher.get_flag("reverse"))
            // Set wrap mode in view
            .set_wrap_mode(!matcher.get_flag("wrap"))
            // Set output in view
            .set_output_mode(output_mode)
            // Set diff(watch diff) in view
            .set_diff_mode(diff_mode)
            .set_diff_mode_width(diff_mode_width)
            .set_only_diffline(matcher.get_flag("diff_output_only"))
            .set_ignore_spaceblock(matcher.get_flag("ignore_spaceblock"))
            // Set enable summary char
            .set_enable_summary_char(enable_summary_char)
            .set_show_ui(!matcher.get_flag("no_title"))
            .set_show_help_banner(!matcher.get_flag("no_help_banner"));

        // Set logfile
        if let Some(logfile) = logfile {
            view = view.set_logfile(logfile.to_string());
        }

        // Set after_command
        if let Some(after_command) = after_command {
            view = view.set_after_command(after_command.to_string());
            view = view.set_after_command_result_write_file(after_command_result_write_file);
        }

        // start app.
        let _res = view.start(tx, rx, load_results);
    } else {
        // is batch mode
        let mut batch = batch::Batch::new(rx, diff_modes)
            .set_beep(matcher.get_flag("beep"))
            .set_color(matcher.get_flag("color"))
            .set_exit_on_change(exit_on_change)
            .set_output_mode(output_mode)
            .set_diff_mode(diff_mode)
            .set_line_number(matcher.get_flag("line_number"))
            .set_reverse(matcher.get_flag("reverse"))
            .set_only_diffline(matcher.get_flag("diff_output_only"))
            .set_ignore_spaceblock(matcher.get_flag("ignore_spaceblock"));

        // Set logfile
        if let Some(logfile) = logfile {
            batch = batch.set_logfile(logfile.to_string());
        }

        // Set after_command
        if let Some(after_command) = after_command {
            batch = batch.set_after_command(after_command.to_string());
            batch = batch.set_after_command_result_write_file(after_command_result_write_file);
        }

        // start batch.
        let _res = batch.run();
    }
}
