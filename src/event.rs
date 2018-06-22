use cmd::Cmd;

pub enum Event {
    OutputUpdate(Cmd),
    Input(i32),
    Exit,
}