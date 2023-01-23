use std::fmt;

use rosc::OscMessage;

#[derive(Debug)]
pub enum TuioError {
    EmptyMessageError(OscMessage),
    MissingSourceError(OscMessage),
    MissingIdError(OscMessage),
}

impl fmt::Display for TuioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TuioError::EmptyMessageError(msg) => write!(f, "empty message arguments at: {:?}", msg),
            TuioError::MissingSourceError(msg) => write!(f, "missing source name at: {:?}", msg),
            TuioError::MissingIdError(msg) => write!(f, "missing session ID(s) at: {:?}", msg),
        }
    }
}