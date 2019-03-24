// module
use regex::bytes::{Matches, Regex};

pub const ANSI_RE: &str =
    r"[\x1b\x9b]\[[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]";

lazy_static! {
    pub static ref ANSI_REGEX: Regex = Regex::new(ANSI_RE).unwrap();
}

pub fn get_ansi_iter(text: &[u8]) -> Matches {
    ANSI_REGEX.find_iter(text)
}
