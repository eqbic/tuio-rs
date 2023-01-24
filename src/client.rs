use std::{time::{Instant, Duration}, net::{SocketAddr, IpAddr, Ipv4Addr, UdpSocket}, thread, sync::{atomic::{AtomicBool, Ordering, AtomicI32}, Arc}, collections::HashSet, panic::UnwindSafe};

use indexmap::IndexMap;
use rosc::{OscPacket, OscMessage, OscType};

use crate::{cursor::{Cursor, Point, Velocity}, object::Object, blob::Blob, errors::TuioError, listener::{self, Listener}, dispatcher::Dispatch};

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
    current_time: Duration,
    object_map: IndexMap<i32, Object>,
    frame_objects: Vec<(i32, i32, f32, f32, f32, f32, f32, f32, f32, f32)>,
    cursor_map: IndexMap<i32, Cursor>,
    blob_map: IndexMap<i32, Blob>,
    source_list: IndexMap<String, u32>,
    source_id: u32,
    source_name: String,
    source_address: SocketAddr,
    osc_receiver: O,
    listener_list: Vec<Box<dyn Listener>>,
    local_receiver: bool
}

fn unwrap_object_args(args: &[OscType]) -> Option<(i32, i32, f32, f32, f32, f32, f32, f32, f32, f32)> {
    Some((args[1].clone().int()?, args[2].clone().int()?, args[3].clone().float()?, args[4].clone().float()?, args[5].clone().float()?, args[6].clone().float()?, args[7].clone().float()?, args[8].clone().float()?, args[9].clone().float()?, args[10].clone().float()?))
}

fn unwrap_cursor_args(args: &[OscType]) -> Option<(i32, i32, f32, f32, f32, f32, f32, f32, f32, f32)> {
    Some((args[0].clone().int()?, args[0].clone().int()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?))
}

fn unwrap_blob_args(args: &[OscType]) -> Option<(i32, i32, f32, f32, f32, f32, f32, f32, f32, f32)> {
    Some((args[0].clone().int()?, args[0].clone().int()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?, args[0].clone().float()?))
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
            current_time: Duration::default(),
            source_list: IndexMap::new(),
            source_id: 0,
            source_name: String::default(),
            local_receiver: true,
            object_map: IndexMap::new(),
            frame_objects: Vec::new(),
            cursor_map: IndexMap::new(),
            blob_map: IndexMap::new(),
            listener_list: Vec::new(),
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

                                let mut removed_ids = Vec::with_capacity(self.object_map.len());

                                self.object_map.retain(|key, _| {
                                    let keep = to_keep.contains(key);
                                    if !keep {
                                        removed_ids.push(*key);
                                    }
                                    keep
                                });

                                self.remove_objects(&removed_ids);
                            },
                            "set" => {
                                if message.args.len() == 11 {
                                    if let Some(args) = unwrap_object_args(&message.args)
                                    {
                                        self.frame_objects.push(args);
                                    }
                                    else {
                                        eprintln!("{}", TuioError::MissingArgumentsError(message));
                                    }
                                }
                                else {
                                    eprintln!("{}", TuioError::MissingArgumentsError(message));
                                }
                            },
                            "fseq" => {
                                if let Some(OscType::Int(fseq)) = message.args.get(1) {
                                    let mut late_frame = false;

                                    if fseq > &0 {
                                        let current_frame = self.current_frame.load(Ordering::SeqCst);
                                        if fseq > &current_frame {
                                            self.current_time = self.instant.elapsed();
                                        }
                                        
                                        if fseq >= &current_frame || current_frame - fseq > 100 {
                                            self.current_frame.store(*fseq, Ordering::SeqCst);
                                        }
                                        else {
                                            late_frame = true;
                                        }
                                    }

                                    if !late_frame {
                                        for (session_id, class_id, x_pos, y_pos, angle, x_vel, y_vel, angular_speed, acceleration, angular_acceleration) in self.frame_objects.drain(..) {
                                            // To do: notify add and update
                                            self.object_map.entry(session_id)
                                            .and_modify(|entry| entry.update_values(class_id, Point { x: x_pos, y: y_pos }, angle, Velocity{x: x_vel, y: y_vel}, angular_speed, acceleration, angular_acceleration))
                                            .or_insert(Object::new(self.current_time, session_id, class_id, Point { x: x_pos, y: y_pos }, angle).with_movement(Velocity{x: x_vel, y: y_vel}, angular_speed, acceleration, angular_acceleration));
                                        }
                                    }
                                }
                                else {
                                    eprintln!("{}", TuioError::MissingArgumentsError(message));
                                }
                            },
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

impl<O: OscReceiver> Dispatch for Client<O> {
    fn add_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.listener_list.push(Box::new(listener))
    }

    fn remove_listener<L: Listener + 'static>(&mut self, listener: L) {
        let listener: Box<dyn Listener> = Box::new(listener);
        self.listener_list.retain(|x| x == &listener)
    }

    fn remove_all_listeners(&mut self) {
        self.listener_list.clear();
    }

    fn get_objects(&self) -> Vec<&Object> {
        self.object_map.values().collect()
    }

    fn get_object_count(&self) -> usize {
        self.object_map.len()
    }

    fn get_cursors(&self) -> Vec<&Cursor> {
        self.cursor_map.values().collect()
    }

    fn get_cursor_count(&self) -> usize {
        self.cursor_map.len()
    }

    fn get_blobs(&self) -> Vec<&Blob> {
        self.blob_map.values().collect()
    }

    fn get_blob_count(&self) -> usize {
        self.blob_map.len()
    }

    fn get_object(&self, session_id: i32) -> Option<&Object> {
        self.object_map.get(&session_id)
    }

    fn get_cursor(&self, session_id: i32) -> Option<&Cursor> {
        self.cursor_map.get(&session_id)
    }

    fn get_blob(&self, session_id: i32) -> Option<&Blob> {
        self.blob_map.get(&session_id)
    }

    fn get_listeners(&mut self) -> &mut Vec<Box<dyn Listener>> {
        &mut self.listener_list
    }
}