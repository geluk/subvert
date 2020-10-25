use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SubvertError {
    ParseError(String),
}

impl Error for SubvertError {}

impl fmt::Display for SubvertError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubvertError::ParseError(msg) => write!(fmt, "{}", msg),
        }
    }
}
