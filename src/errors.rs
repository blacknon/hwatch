use std::{error::Error, fmt::Display};

pub type Result<T> = error_stack::Result<T, HwatchError>;

#[derive(Debug)]
pub enum HwatchError {
    ConfigError,
    TuiError(String),
}

impl Display for HwatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError => write!(f, "Config Error"),
            Self::TuiError(inner) => write!(f, "TUI error: {inner}"),
        }
    }
}

impl Error for HwatchError {}
