use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc,
    }, error::Error,
};

use rosc::{OscPacket};

use crate::errors::OscReceiverError;

/// Base trait to implement receiving OSC over various transport methods
pub trait OscReceiver<P, E: Error> {
    /// Returns a true if the connection is established
    fn is_connected(&self) -> bool;

    /// Establishes connection
    fn connect(&self) -> Result<(), std::io::Error>;

    /// Stops connection
    fn disconnect(&self);

    /// Receives a single OSC packet.
    fn recv(&self) -> Result<P, E>;
}

pub struct UdpReceiver {
    socket: Arc<UdpSocket>
}

impl UdpReceiver {
    /// Creates an [UdpReceiver] binded to the port 3333
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_port(3333)
    }
    
    /// Creates an [UdpReceiver] binded to the provided port
    pub fn from_port(port: u16) -> Result<Self, std::io::Error> {
        Ok(Self {
            socket: Arc::new(UdpSocket::bind(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                port,
            ))?)
        })
    }
}

pub type RoscReceiver = dyn OscReceiver<OscPacket, OscReceiverError> + Send + Sync;

impl OscReceiver<OscPacket, OscReceiverError> for UdpReceiver {
    fn connect(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn disconnect(&self) {}

    /// Always returns true because UDP is connectionless
    fn is_connected(&self) -> bool {
        true
    }

    fn recv(&self) -> Result<OscPacket, OscReceiverError> {
        let mut buf = [0u8; rosc::decoder::MTU];

        let (size, _) = self.socket.recv_from(&mut buf).map_err(OscReceiverError::Receive)?;
        let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).map_err(OscReceiverError::Decode)?;

        Ok(packet)
    }
}