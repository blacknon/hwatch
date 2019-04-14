use cmd::Result;

pub enum Event {
    OutputUpdate(Result),
    Input(i32),
    Signal(i32),
    Exit,
}
