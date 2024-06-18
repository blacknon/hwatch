// Copyright (c) 2024 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

// v0.3.16
// TODO(blacknon): Enterキーでfilter modeのキーワード移動をできるようにする
// TODO(blacknon): filter modeのハイライト表示をどのoutput modeでもできるようにする(とりあえずcolor mode enable時はansi codeをパース前にいじる感じにすれば良さそう？)
// TODO(blacknon): filter modeのハイライト表示の色を環境変数で定義できるようにする
// TODO(blacknon): コマンドが終了していなくても、インターバル間隔でコマンドを実行する
//                 (パラレルで実行してもよいコマンドじゃないといけないよ、という機能か。投げっぱなしにしてintervalで待つようにするオプションを付ける)
// TODO(blacknon): watchをモダンよりのものに変更する
// TODO(blacknon): diff modeをさらに複数用意し、選択・切り替えできるdiffをオプションから指定できるようにする(watchをold-watchにして、モダンなwatchをデフォルトにしたり)
// TODO(blacknon): Windowsのバイナリをパッケージマネジメントシステムでインストール可能になるよう、Releaseでうまいこと処理をする
// TODO(blacknon): watchウィンドウの表示を折り返しだけではなく、横方向にスクロールして出力するモードも追加する
// TODO(blacknon): UTF-8以外のエンコードでも動作するよう対応する(エンコード対応)
// TODO(blacknon): https://github.com/blacknon/hwatch/issues/101
//                 - ログを読み込ませて、そのまま続きの処理を行わせる機能の追加
// TODO(blacknon): key入力の処理をcrosstermからtermwizに変更した場合、Macだとマウス操作時にコントロールキャラが入力されてしまう問題が解決するか検証

// v0.3.17
// TODO(blacknon): ...

// v1.0.0
// TODO(blacknon): vimのように内部コマンドを利用した表示切り替え・出力結果の編集機能を追加する
// TODO(blacknon): 任意時点間のdiffが行えるようにする.
// TODO(blacknon): filtering時に、`指定したキーワードで差分が発生した場合のみ`を対象にするような機能を追加する(command mode option)
// TODO(blacknon): Rustのドキュメンテーションコメントを追加していく
// TODO(blacknon): マニュアル(manのデータ)を自動作成させる
//                 https://github.com/rust-cli/man
// TODO(blacknon): エラーなどのメッセージ表示領域の作成

// crate
extern crate ansi_parser;
extern crate ansi_term;
extern crate async_std;
extern crate config;
extern crate chrono;
extern crate chardetng;
extern crate crossbeam_channel;
extern crate crossterm;
extern crate ctrlc;
extern crate encoding_rs;
extern crate flate2;
extern crate futures;
extern crate heapless;
extern crate question;
extern crate regex;
extern crate serde;
extern crate shell_words;
extern crate similar;
extern crate termwiz;
extern crate ratatui as tui;

// macro crate
#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

// modules
use clap::{Arg, ArgAction, Command, ValueHint, builder::ArgPredicate};
use question::{Answer, Question};
use std::env::args;
use std::path::Path;
use std::sync::{Arc, RwLock};
use crossbeam_channel::unbounded;
use std::thread;
use std::time::Duration;
use common::DiffMode;

// local modules
mod ansi;
mod app;
mod batch;
mod common;
mod errors;
mod event;
mod exec;
mod exit;
mod header;
mod help;
mod history;
mod keymap;
mod output;
mod view;
mod watch;

// const
pub const DEFAULT_INTERVAL: f64 = 2.0;
pub const DEFAULT_TAB_SIZE: u16 = 4;
pub const HISTORY_WIDTH: u16 = 25;
pub const SHELL_COMMAND_EXECCMD: &str = "{COMMAND}";
pub const HISTORY_LIMIT: &str = "5000";
type Interval = Arc<RwLock<f64>>;

// const at Windows
#[cfg(windows)]
const SHELL_COMMAND: &str = "cmd /C";

// const at not Windows
#[cfg(not(windows))]
const SHELL_COMMAND: &str = "sh -c";

/// Parse args and options function.
fn build_app() -> clap::Command {
    // get own name
    let _program = args()
        .next()
        .and_then(|s| {
            std::path::PathBuf::from(s)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap();

    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .trailing_var_arg(true)
        .author(crate_authors!())

        // -- command --
        .arg(
            Arg::new("command")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .num_args(0..)
                .value_hint(ValueHint::CommandWithArguments)
                .required(true),
        )

        // -- flags --
        // Enable batch mode option
        //     [-b,--batch]
        .arg(
            Arg::new("batch")
                .help("output exection results to stdout")
                .short('b')
                .action(ArgAction::SetTrue)
                .long("batch"),
        )
        // Beep option
        //     [-B,--beep]
        .arg(
            Arg::new("beep")
                .help("beep if command has a change result")
                .short('B')
                .action(ArgAction::SetTrue)
                .long("beep"),
        )
        .arg(
            Arg::new("border")
                .help("Surround each pane with a border frame")
                .action(ArgAction::SetTrue)
                .long("border"),
        )
        .arg(
            Arg::new("with_scrollbar")
                .help("When the border option is enabled, display scrollbar on the right side of watch pane.")
                .action(ArgAction::SetTrue)
                .long("with-scrollbar"),
        )
        // mouse option
        //     [--mouse]
        .arg(
            Arg::new("mouse")
                .help("enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.")
                .action(ArgAction::SetTrue)
                .long("mouse"),
        )
        // Enable ANSI color option
        //     [-c,--color]
        .arg(
            Arg::new("color")
                .help("interpret ANSI color and style sequences")
                .short('c')
                .action(ArgAction::SetTrue)
                .long("color"),
            )
        // Enable Reverse mode option
        //     [-r,--reverse]
        .arg(
            Arg::new("reverse")
                .help("display text upside down.")
                .short('r')
                .action(ArgAction::SetTrue)
                .long("reverse"),
        )
        // Compress data in memory option.
        //     [-C,--compress]
        .arg(
            Arg::new("compress")
                .help("Compress data in memory. Note: If the output of the command is small, you may not get the desired effect.")
                .short('C')
                .action(ArgAction::SetTrue)
                .long("compress"),
        )
        // exec flag.
        //     [--no-title]
        .arg(
            Arg::new("no_title")
                .help("hide the UI on start. Use `t` to toggle it.")
                .long("no-title")
                .action(ArgAction::SetTrue)
                .short('t'),
        )
        // Enable line number mode option
        //   [--line-number,-N]
        .arg(
            Arg::new("line_number")
                .help("show line number")
                .short('N')
                .action(ArgAction::SetTrue)
                .long("line-number"),
        )
        // exec flag.
        //     [--no-help-banner]
        .arg(
            Arg::new("no_help_banner")
                .help("hide the \"Display help with h key\" message")
                .long("no-help-banner")
                .action(ArgAction::SetTrue),
        )
        // exec flag.
        //     [-x,--exec]
        .arg(
            Arg::new("exec")
                .help("Run the command directly, not through the shell. Much like the `-x` option of the watch command.")
                .short('x')
                .action(ArgAction::SetTrue)
                .long("exec"),

        )
        // output only flag.
        //     [-O,--diff-output-only]
        .arg(
            Arg::new("diff_output_only")
                .help("Display only the lines with differences during `line` diff and `word` diff.")
                .short('O')
                .long("diff-output-only")
                .requires("differences")
                .action(ArgAction::SetTrue),

        )

        // -- options --
        // Option to specify the command to be executed when the output fluctuates.
        //     [-A,--aftercommand]
        .arg(
            Arg::new("after_command")
                .help("Executes the specified command if the output changes. Information about changes is stored in json format in environment variable ${HWATCH_DATA}.")
                .short('A')
                .long("aftercommand")
                .value_hint(ValueHint::CommandString)
                .action(ArgAction::Append)
        )
        // Logging option
        //   [--logfile,-l] /path/to/logfile
        // ex.)
        //      {timestamp: "...", command: "....", output: ".....", ...}
        //      {timestamp: "...", command: "....", output: ".....", ...}
        //      {timestamp: "...", command: "....", output: ".....", ...}
        .arg(
            Arg::new("logfile")
                .help("logging file")
                .short('l')
                .long("logfile")
                .num_args(0..=1)
                .value_hint(ValueHint::FilePath)
                .action(ArgAction::Append),
        )
        // shell command
        //   [--shell,-s] command
        .arg(
            Arg::new("shell_command")
                .help("shell to use at runtime. can  also insert the command to the location specified by {COMMAND}.")
                .short('s')
                .long("shell")
                .action(ArgAction::Append)
                .value_hint(ValueHint::CommandString)
                .default_value(SHELL_COMMAND),
        )
        // Interval option
        //   [--interval,-n] second(default:2)
        .arg(
            Arg::new("interval")
                .help("seconds to wait between updates")
                .short('n')
                .long("interval")
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(f64))
                .default_value("2"),
        )
        // set limit option
        //   [--limit,-L] size(default:5000)
        .arg(
            Arg::new("limit")
                .help("Set the number of history records to keep. only work in watch mode. Set `0` for unlimited recording. (default: 5000)")
                .short('L')
                .long("limit")
                .value_parser(clap::value_parser!(u32))
                .default_value(HISTORY_LIMIT),
        )
        // tab size set option
        //   [--tab_size] size(default:4)
        .arg(
            Arg::new("tab_size")
                .help("Specifying tab display size")
                .long("tab-size")
                .value_parser(clap::value_parser!(u16))
                .action(ArgAction::Append)
                .default_value("4"),
        )
        // Enable diff mode option
        //   [--differences,-d] [none, watch, line, word]
        .arg(
            Arg::new("differences")
                .help("highlight changes between updates")
                .long("differences")
                .short('d')
                .num_args(0..=1)
                .value_parser(["none", "watch", "line", "word"])
                .default_missing_value("watch")
                .default_value_ifs([("differences", ArgPredicate::IsPresent, None)])
                .action(ArgAction::Append),
        )
        // Set output mode option
        //   [--output,-o] [output, stdout, stderr]
        .arg(
            Arg::new("output")
                .help("Select command output.")
                .short('o')
                .long("output")
                .num_args(0..=1)
                .value_parser(["output", "stdout", "stderr"])
                .default_value("output")
                .action(ArgAction::Append),
        )
        //
        .arg(
            Arg::new("keymap")
                .help("Add keymap")
                .short('K')
                .long("keymap")
                .action(ArgAction::Append),
        )

}

fn get_clap_matcher() -> clap::ArgMatches {
    let env_config = std::env::var("HWATCH").unwrap_or_default();
    let env_args: Vec<&str> = env_config.split_ascii_whitespace().collect();
    let mut os_args = std::env::args_os();
    let mut args: Vec<std::ffi::OsString> = vec![];

    // First argument is the program name
    args.push(os_args.next().unwrap());

    // Environment variables go next so that they can be overridded
    // TODO: Currently, the opposites of command-line options are not
    // yet implemented. E.g., there is no `--no-color` to override
    // `--color` in the HWATCH environment variable.
    args.extend(env_args.iter().map(std::ffi::OsString::from));
    args.extend(os_args);

    build_app().get_matches_from(args)
}

fn main() {
    // Get command args matches
    let matcher = get_clap_matcher();

    // Get options flag
    let batch = matcher.get_flag("batch");
    let compress = matcher.get_flag("compress");

    // Get after command
    let after_command = matcher.get_one::<String>("after_command");

    // Get logfile
    let logfile = matcher.get_one::<String>("logfile");

    // check _logfile directory
    // TODO(blacknon): commonに移す？(ここで直書きする必要性はなさそう)
    if let Some(logfile) = logfile {
        let _log_path = Path::new(logfile);
        let _log_dir = _log_path.parent().unwrap();
        let _cur_dir = std::env::current_dir().expect("cannot get current directory");
        let _abs_log_path = _cur_dir.join(_log_path);
        let _abs_log_dir = _cur_dir.join(_log_dir);

        // check _log_path exist
        if _abs_log_path.exists() {
            println!("file {_abs_log_path:?} is exists.");
            let answer = Question::new("Log to the same file?")
                .default(Answer::YES)
                .show_defaults()
                .confirm();

            if answer != Answer::YES {
                std::process::exit(1);
            }
        }

        // check _log_dir exist
        if !_abs_log_dir.exists() {
            println!("directory {_abs_log_dir:?} is not exists.");
            std::process::exit(1);
        }
    }

    // Create channel
    let (tx, rx) = unbounded();

    // interval
    let override_interval: f64 = *matcher.get_one::<f64>("interval").unwrap_or(&DEFAULT_INTERVAL);
    let interval = Interval::new(override_interval.into());

    // history limit
    let default_limit:u32 = HISTORY_LIMIT.parse().unwrap();
    let limit = matcher.get_one::<u32>("limit").unwrap_or(&default_limit);

    // tab size
    let tab_size = *matcher.get_one::<u16>("tab_size").unwrap_or(&DEFAULT_TAB_SIZE);

    // output mode
    let output_mode = match matcher.get_one::<String>("output").unwrap().as_str() {
        "output" => common::OutputMode::Output,
        "stdout" => common::OutputMode::Stdout,
        "stderr" => common::OutputMode::Stderr,
        _ => common::OutputMode::Output,
    };

    // diff mode
    let diff_mode = if matcher.contains_id("differences") {
        match matcher.get_one::<String>("differences").unwrap().as_str() {
            "none" => DiffMode::Disable,
            "watch" => DiffMode::Watch,
            "line" => DiffMode::Line,
            "word" => DiffMode::Word,
            _ => DiffMode::Disable,
        }
    } else {
        DiffMode::Disable
    };

    // Get Add keymap
    let keymap_options: Vec<&str> = matcher.get_many::<String>("keymap")
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

    // Start Command Thread
    {
        let m = matcher.clone();
        let tx = tx.clone();
        let shell_command = m.get_one::<String>("shell_command").unwrap().to_string();
        let command: Vec<_> = m.get_many::<String>("command").unwrap().into_iter().map(|s| s.clone()).collect();
        let is_exec = m.get_flag("exec");
        let interval = interval.clone();
        let _ = thread::spawn(move || loop {
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

            // Exec command
            exe.exec_command();

            let sleep_interval = *interval.read().unwrap();
            std::thread::sleep(Duration::from_secs_f64(sleep_interval));
        });
    }

    // check batch mode
    if !batch {
        // is watch mode
        // Create view
        let mut view = view::View::new(interval.clone())
            // Set interval on view.header
            .set_interval(interval)
            .set_tab_size(tab_size)
            .set_limit(*limit)
            .set_beep(matcher.get_flag("beep"))
            .set_border(matcher.get_flag("border"))
            .set_scroll_bar(matcher.get_flag("with_scrollbar"))
            .set_mouse_events(matcher.get_flag("mouse"))

            // set keymap
            .set_keymap(keymap)

            // Set color in view
            .set_color(matcher.get_flag("color"))

            // Set line number in view
            .set_line_number(matcher.get_flag("line_number"))

            // Set reverse mode in view
            .set_reverse(matcher.get_flag("reverse"))

            // Set output in view
            .set_output_mode(output_mode)

            // Set diff(watch diff) in view
            .set_diff_mode(diff_mode)
            .set_only_diffline(matcher.get_flag("diff_output_only"))

            .set_show_ui(!matcher.get_flag("no_title"))
            .set_show_help_banner(!matcher.get_flag("no_help_banner"));

        // Set logfile
        if let Some(logfile) = logfile {
            view = view.set_logfile(logfile.to_string());
        }

        // Set after_command
        if let Some(after_command) = after_command {
            view = view.set_after_command(after_command.to_string());
        }

        // start app.
        let _res = view.start(tx, rx);
    } else {
        // is batch mode
        let mut batch = batch::Batch::new(tx, rx)
            .set_beep(matcher.get_flag("beep"))
            .set_output_mode(output_mode)
            .set_diff_mode(diff_mode)
            .set_line_number(matcher.get_flag("line_number"))
            .set_reverse(matcher.get_flag("reverse"))
            .set_only_diffline(matcher.get_flag("diff_output_only"));

        // Set logfile
        if let Some(logfile) = logfile {
            batch = batch.set_logfile(logfile.to_string());
        }

        // Set after_command
        if let Some(after_command) = after_command {
            batch = batch.set_after_command(after_command.to_string());
        }

        // start batch.
        let _res = batch.run();
    }
}
