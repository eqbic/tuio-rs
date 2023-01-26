use std::fmt;

use rosc::OscMessage;

#[derive(Debug)]
pub enum TuioError {
    UnknownAddressError(OscMessage),
    UnknownMessageTypeError(OscMessage),
    EmptyMessageError(OscMessage),
    MissingSourceError(OscMessage),
    MissingArgumentsError(OscMessage),
    WrongArgumentTypeError(OscMessage, u8),
}

impl fmt::Display for TuioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TuioError::UnknownAddressError(msg) => write!(f, "unknown address: {:?}", msg.addr),
            TuioError::UnknownMessageTypeError(msg) => write!(f, "unknown message type: {:?}", msg),
            TuioError::EmptyMessageError(msg) => write!(f, "empty message at: {:?}", msg),
            TuioError::MissingSourceError(msg) => write!(f, "missing source name at: {:?}", msg),
            TuioError::MissingArgumentsError(msg) => write!(f, "missing one or more arguments at: {:?}", msg),
            TuioError::WrongArgumentTypeError(msg, index) => write!(f, "wrong argument type at index {} in: {:?}", index, msg),
        }
    }
}