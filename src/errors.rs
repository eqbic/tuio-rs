use std::{fmt, io, error};

use rosc::{OscMessage, OscBundle, OscPacket, OscError};

#[derive(Debug)]
pub enum TuioError {
    UnknownAddress(OscMessage),
    UnknownMessageType(OscMessage),
    EmptyMessage(OscMessage),
    MissingSource(OscMessage),
    MissingArguments(OscMessage),
    WrongArgumentType(OscMessage, u8),
    IncompleteBundle(OscBundle),
    NotABundle(OscPacket),
}

impl fmt::Display for TuioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TuioError::UnknownAddress(msg) => write!(f, "unknown address: {:?}", msg.addr),
            TuioError::UnknownMessageType(msg) => write!(f, "unknown message type: {:?}", msg),
            TuioError::EmptyMessage(msg) => write!(f, "empty message at: {:?}", msg),
            TuioError::MissingSource(msg) => write!(f, "missing source name at: {:?}", msg),
            TuioError::MissingArguments(msg) => write!(f, "missing one or more arguments at: {:?}", msg),
            TuioError::WrongArgumentType(msg, index) => write!(f, "wrong argument type at index {} in: {:?}", index, msg),
            TuioError::IncompleteBundle(bundle) => write!(f, "missing one or more mandatory messages in: {:?}", bundle),
            TuioError::NotABundle(packet) => write!(f, "OSC packet is not a bundle: {:?}", packet),
        }
    }
}

#[derive(Debug)]
pub enum OscReceiverError {
    Connect(io::Error),
    AlreadyConnected(),
    Receive(io::Error),
    Decode(OscError),
}

impl fmt::Display for OscReceiverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OscReceiverError::AlreadyConnected() => write!(f, "OscReceiver is already connected"),
            OscReceiverError::Connect(msg) => write!(f, "error connecting OscReceiver: {}", msg),
            OscReceiverError::Receive(msg) => write!(f, "error receiving OSC packet: {}", msg),
            OscReceiverError::Decode(msg) => write!(f, "error decoding OSC packet: {}", msg),
        }
    }
}

impl error::Error for OscReceiverError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            OscReceiverError::AlreadyConnected() => None,
            OscReceiverError::Connect(err) => Some(err),
            OscReceiverError::Receive(err) => Some(err),
            OscReceiverError::Decode(err) => Some(err)
        }
    }
}
