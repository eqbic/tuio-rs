use std::{net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr}, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread};

use rosc::OscPacket;

/// Base trait to implement receiving OSC over various transport methods
pub trait OscReceiver {
    /// Returns a true if the connection is established
    fn is_connected(&self) -> bool;

    /// Establishes connection
    fn connect(&self);

    /// Stops connection
    fn disconnect(&self);

    fn from_port(port: u16) -> Result<Self, std::io::Error> where Self: Sized;
}

pub struct UdpReceiver {
    socket: Arc<UdpSocket>,
    listen: Arc<AtomicBool>
}

impl UdpReceiver {
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_port(3333)
    }

    pub fn from_port(port: u16) -> Result<Self, std::io::Error> {
        Ok(Self {socket: Arc::new(UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))?), listen: Arc::new(AtomicBool::new(false))})
    }

    fn handle_packet(packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => {
                println!("OSC address: {}", msg.addr);
                println!("OSC arguments: {:?}", msg.args);
            }
            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);
            }
        }
    }
}

impl OscReceiver for UdpReceiver {
    fn connect(&self) {
        let mut buf = [0u8; rosc::decoder::MTU];

        self.listen.store(true, Ordering::Relaxed);
        let listen = Arc::clone(&self.listen);
        let socket = Arc::clone(&self.socket);

        thread::spawn(move || loop {
            if !listen.load(Ordering::Relaxed) {break;}
            match socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    println!("Received packet with size {} from: {}", size, addr);
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    Self::handle_packet(packet);
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            }
        });
    }

    fn disconnect(&self) {
        self.listen.store(false, Ordering::Relaxed);
    }

    /// Always returns true because UDP is connectionless
    fn is_connected(&self) -> bool {
        true
    }

    fn from_port(port: u16) -> Result<Self, std::io::Error> {
        Ok(Self {socket: Arc::new(UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))?), listen: Arc::new(AtomicBool::new(false))})
    }
}

impl Drop for UdpReceiver {
    fn drop(&mut self) {
        self.disconnect();
    }
}