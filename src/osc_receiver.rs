use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use ringbuffer::{ConstGenericRingBuffer, RingBufferWrite, RingBufferRead};
use rosc::OscPacket;

/// Base trait to implement receiving OSC over various transport methods
pub trait OscReceiver {
    /// Returns a true if the connection is established
    fn is_connected(&self) -> bool;

    /// Establishes connection
    fn connect(&self);

    /// Stops connection
    fn disconnect(&self);

    fn receive(&self) -> Vec<OscPacket>;
}

pub struct UdpReceiver {
    socket: Arc<UdpSocket>,
    listen: Arc<AtomicBool>,
    packet_buffer: Arc<Mutex<ConstGenericRingBuffer<OscPacket, 16>>>
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
            ))?),
            listen: Arc::new(AtomicBool::new(false)),
            packet_buffer: Default::default()
        })
    }
}

impl OscReceiver for UdpReceiver {
    fn connect(&self) {
        let mut buf = [0u8; rosc::decoder::MTU];

        self.listen.store(true, Ordering::Relaxed);
        let listen = Arc::clone(&self.listen);
        let socket = Arc::clone(&self.socket);

        let packet_buffer = self.packet_buffer.clone();

        thread::spawn(move || loop {
            if !listen.load(Ordering::Relaxed) {
                break;
            }
            
            match socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    println!("Received packet with size {} from: {}", size, addr);
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    packet_buffer.lock().unwrap().push(packet);
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

    /// Returns a vector of all [OscPacket] received since the last call
    fn receive(&self) -> Vec<OscPacket> {
        self.packet_buffer.lock().unwrap().drain().collect()
    }
}

impl Drop for UdpReceiver {
    fn drop(&mut self) {
        self.disconnect();
    }
}
