// crate
extern crate chrono;

// module
use self::chrono::Local;

pub fn now_str() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S").to_string();
}
