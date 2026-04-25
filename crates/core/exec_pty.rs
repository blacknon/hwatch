// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

#[cfg(unix)]
use nix::pty::{openpty, OpenptyResult, Winsize};
#[cfg(unix)]
use nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg};

#[cfg(unix)]
pub(super) fn create_raw_pty() -> Result<OpenptyResult, nix::Error> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let result = openpty(Some(&winsize), None)?;

    let mut termios = tcgetattr(&result.slave)?;
    cfmakeraw(&mut termios);
    tcsetattr(&result.slave, SetArg::TCSANOW, &termios)?;

    Ok(result)
}
