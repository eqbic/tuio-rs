use std::{time::Instant, net::{SocketAddr, IpAddr, Ipv4Addr, UdpSocket}, thread, sync::{atomic::{AtomicBool, Ordering, AtomicI32}, Arc}, collections::HashSet};

use indexmap::IndexMap;
use rosc::{OscPacket, OscMessage, OscType};

use crate::{cursor::Cursor, object::Object, blob::Blob, errors::TuioError};

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

pub struct Client<O: OscReceiver> {
    frame_cursor: Vec<Cursor>,
    alive_cursor_id_list: Vec<u32>,
    current_frame: AtomicI32,
    instant: Instant,
    object_map: IndexMap<i32, Object>,
    cursor_map: IndexMap<i32, Cursor>,
    blob_map: IndexMap<i32, Blob>,
    source_list: IndexMap<String, u32>,
    source_id: u32,
    source_name: String,
    source_address: SocketAddr,
    osc_receiver: O,
    local_receiver: bool
}

impl<O: OscReceiver> Client<O>{
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_port(3333)
    }

    pub fn from_port(port: u16) -> Result<Self, std::io::Error> {
        Ok(Self {instant: Instant::now(),
            source_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port),
            osc_receiver: O::from_port(port)?,
            frame_cursor: Vec::new(),
            alive_cursor_id_list: Vec::new(),
            current_frame: (-1).into(),
            source_list: IndexMap::new(),
            source_id: 0,
            source_name: "todo!()".to_string(),
            local_receiver: true,
            object_map: IndexMap::new(),
            cursor_map: IndexMap::new(),
            blob_map: IndexMap::new(),
        })
    }

    pub fn connect(&self) {
        self.osc_receiver.connect();
    }

    pub fn disconnect(&self) {
        self.osc_receiver.disconnect();
    }

    fn get_source_id(&mut self, name: &String) -> u32 {
        match self.source_list.get(name) {
            Some(id) => *id,
            None => {
                let index = self.source_list.len() as u32;
                self.source_list.insert(name.to_string(), index);
                index
            },
        }
    }

    fn process_osc_message(&mut self, message: OscMessage) {
        match message.addr.as_str() {
            "/tuio/2Dobj" => {
                match message.args.first() {
                    Some(OscType::String(arg)) => {
                        match arg.as_str() {
                            "source" => {
                                if let Some(OscType::String(source_name)) = message.args.get(1) {
                                    self.source_id = self.get_source_id(source_name);
                                }
                                else {
                                    eprintln!("{}", TuioError::MissingSourceError(message));
                                }
                            },
                            "alive" => {
                                let to_keep: HashSet<i32> = HashSet::from_iter(message.args.into_iter().skip(1).filter_map(|e| e.int()));
                                self.object_map.retain(|key, _| to_keep.contains(key));
                            },
                            "set" => {},
                            "fseq" => {},
                            _ => ()
                        }
                    }
                    None => eprintln!("{}", TuioError::EmptyMessageError(message)),
                    _ => ()
                }
            }
            _ => {
                println!("Unknow address: {}", message.addr);
            }
        }
    }

    fn process_osc_packet(&mut self, packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => {
                println!("OSC address: {}", msg.addr);
                println!("OSC arguments: {:?}", msg.args);
                
                self.process_osc_message(msg);
            }
            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);

                for message in bundle.content {
                    self.process_osc_packet(message);
                }
            }
        }
    }
}