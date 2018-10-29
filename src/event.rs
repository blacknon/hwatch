use cmd::Result;
// use nix::sys::signal;

pub enum Event {
    OutputUpdate(Result),
    Input(i32),
    Signal(i32),
    Exit,
}