// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

use crossbeam_channel::Sender;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::Duration;
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
use nix::fcntl::{fcntl, FcntlArg::*, OFlag};
#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
use std::io::stdin;

use super::View;
use crate::app::App;
use crate::event::AppEvent;

impl View {
    pub(super) fn apply_to_app(
        &self,
        app: &mut App<'_>,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        app.set_keymap(self.keymap.clone());
        app.set_after_command(self.after_command.clone());
        app.set_after_command_shell_command(self.after_command_shell_command.clone());
        app.set_after_command_result_write_file(self.after_command_result_write_file);
        app.set_watch_diff_colors(self.watch_diff_fg, self.watch_diff_bg);

        if cfg!(target_os = "windows") {
            execute!(terminal.backend_mut(), EnableMouseCapture)?;
        }
        app.set_mouse_events(self.mouse_events);

        app.set_limit(self.limit);
        app.set_beep(self.beep);
        app.set_exit_on_change(self.exit_on_change);
        app.set_border(self.border);
        app.set_scroll_bar(self.scroll_bar);
        app.set_logpath(self.log_path.clone());
        app.set_ansi_color(self.color);
        app.show_history(self.show_ui);
        app.show_ui(self.show_ui);
        app.show_help_banner(self.show_help_banner);
        app.set_tab_size(self.tab_size);
        app.set_line_number(self.line_number);
        app.set_reverse(self.reverse);
        app.set_wrap_mode(self.wrap);
        app.set_output_mode(self.output_mode);
        app.set_diff_mode(self.diff_mode);
        app.set_is_only_diffline(self.is_only_diffline);
        app.set_ignore_spaceblock(self.ignore_spaceblock);
        app.set_summary_enabled(self.summary_enabled);
        app.set_enable_summary_char(self.enable_summary_char);

        Ok(())
    }
}

pub(super) fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(backend)?;
    let _ = terminal.clear();
    Ok(terminal)
}

pub(super) fn spawn_input_thread(input_tx: Sender<AppEvent>) {
    let _ = std::thread::spawn(move || {
        #[cfg(any(
            target_os = "freebsd",
            target_os = "linux",
            target_os = "macos",
            target_os = "windows"
        ))]
        loop {
            let _ = send_input(input_tx.clone());
        }
    });
}

pub(super) fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();
}

fn send_input(tx: Sender<AppEvent>) -> io::Result<()> {
    if crossterm::event::poll(Duration::from_millis(100))? {
        #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
        set_nonblocking(true)?;

        let result = crossterm::event::read();

        #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
        set_nonblocking(false)?;

        match result {
            Ok(event) => {
                let _ = tx.send(AppEvent::TerminalEvent(event));
            }
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
pub fn set_nonblocking(is_nonblocking: bool) -> nix::Result<()> {
    let stdin = stdin();
    let flags = fcntl(&stdin, F_GETFL)?;
    let new_flags = next_nonblocking_flags(flags, is_nonblocking);
    fcntl(&stdin, F_SETFL(new_flags))?;

    Ok(())
}

#[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
pub(super) fn next_nonblocking_flags(current_flags: i32, is_nonblocking: bool) -> OFlag {
    let current_flags = OFlag::from_bits_truncate(current_flags);

    if is_nonblocking {
        current_flags | OFlag::O_NONBLOCK
    } else {
        current_flags & !OFlag::O_NONBLOCK
    }
}

#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
    use super::next_nonblocking_flags;
    #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
    use nix::fcntl::OFlag;

    #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
    #[test]
    fn next_nonblocking_flags_enables_nonblocking_without_dropping_other_flags() {
        let current_flags = (OFlag::O_APPEND | OFlag::O_CLOEXEC).bits();
        let next_flags = next_nonblocking_flags(current_flags, true);

        assert!(next_flags.contains(OFlag::O_NONBLOCK));
        assert!(next_flags.contains(OFlag::O_APPEND));
        assert!(next_flags.contains(OFlag::O_CLOEXEC));
    }

    #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "macos"))]
    #[test]
    fn next_nonblocking_flags_disables_nonblocking_without_dropping_other_flags() {
        let current_flags = (OFlag::O_APPEND | OFlag::O_NONBLOCK).bits();
        let next_flags = next_nonblocking_flags(current_flags, false);

        assert!(!next_flags.contains(OFlag::O_NONBLOCK));
        assert!(next_flags.contains(OFlag::O_APPEND));
    }
}
