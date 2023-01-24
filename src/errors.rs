use std::fmt;

use rosc::OscMessage;

#[derive(Debug)]
pub enum TuioError {
    EmptyMessageError(OscMessage),
    MissingSourceError(OscMessage),
    MissingArgumentsError(OscMessage),
    WrongArgumentsError(OscMessage),
}

impl fmt::Display for TuioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TuioError::EmptyMessageError(msg) => write!(f, "empty message arguments at: {:?}", msg),
            TuioError::MissingSourceError(msg) => write!(f, "missing source name at: {:?}", msg),
            TuioError::MissingArgumentsError(msg) => write!(f, "missing arguments at: {:?}", msg),
            TuioError::WrongArgumentsError(msg) => write!(f, "wrong argument(s) type at: {:?}", msg),
        }
    }
}