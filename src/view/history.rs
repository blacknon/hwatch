use std::sync::Mutex;
use cmd::Result;

pub struct History {
    pub count: i32,
    pub history: Mutex<Vec<Result>>
}

impl History {
    pub fn new() -> Self {
        Self {
            count: 0,
            history : Mutex::new(vec![])
        }
    }

    pub fn get_latest_history(&mut self) -> Result {
        let mut _result = Result::new();

        let mut _history = self.history.lock().unwrap();
        let _length = _history.len();
        if _length >= 1 {
            _result = _history[_length - 1].clone();
        }
        return _result;
    }

    pub fn append_history(&mut self, _result: Result) {
        let mut history = self.history.lock().unwrap();
        history.push(_result);
        self.count += 1;
    }
}