use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum HwatchError {
    ConfigError,
}

impl Display for HwatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError => write!(f, "Config Error"),
        }
    }
}

impl Error for HwatchError {}
