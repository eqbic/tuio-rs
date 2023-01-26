use std::{time::{Instant, Duration}, net::{SocketAddr, IpAddr, Ipv4Addr, UdpSocket}, thread, sync::{atomic::{AtomicBool, Ordering, AtomicI32}, Arc}, collections::HashSet};

use indexmap::IndexMap;
use rosc::{OscPacket, OscMessage, OscType};

use crate::{cursor::{Cursor, Point, Velocity}, object::Object, blob::Blob, errors::TuioError, listener::{self, Listener}, dispatcher::{Dispatch, Dispatcher}};

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

type ObjectParams = (i32, i32, f32, f32, f32, f32, f32, f32, f32, f32);
#[derive(Default)]
struct SourceCollection {
    object_map: IndexMap<i32, Object>,
    blob_map: IndexMap<i32, Blob>,
    cursor_map: IndexMap<i32, Cursor>
}

pub struct Client<O: OscReceiver> {
    frame_cursor: Vec<Cursor>,
    alive_cursor_id_list: Vec<u32>,
    current_frame: AtomicI32,
    instant: Instant,
    current_time: Duration,
    frame_objects: Vec<ObjectParams>,
    source_list: IndexMap<String, SourceCollection>,
    source_id: u32,
    source_name: String,
    source_address: SocketAddr,
    osc_receiver: O,
    dispatcher: Dispatcher,
    local_receiver: bool
}


fn unwrap_object_args(args: &[OscType]) -> Result<ObjectParams, u8> {
    let args = args.clone();

    Ok((
        args[1].clone().int().ok_or(1)?,
        args[2].clone().int().ok_or(2)?,
        args[3].clone().float().ok_or(3)?,
        args[4].clone().float().ok_or(4)?,
        args[5].clone().float().ok_or(5)?,
        args[6].clone().float().ok_or(6)?,
        args[7].clone().float().ok_or(7)?,
        args[8].clone().float().ok_or(8)?,
        args[9].clone().float().ok_or(9)?,
        args[10].clone().float().ok_or(10)?
    ))
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
            frame_objects: Vec::new(),
            dispatcher: Dispatcher::new(),
        })
    }

    pub fn connect(&self) {
        self.osc_receiver.connect();
    }

    pub fn disconnect(&self) {
        self.osc_receiver.disconnect();
    }

    /// Update frame parameters based on a frame number
    /// 
    /// Returns true if the frame is a new frame
    /// # Argument
    /// * `frame` - the new frame number

    fn update_frame(&mut self, frame: i32) -> bool {
        if frame > 0 {
            let current_frame = self.current_frame.load(Ordering::SeqCst);
            if frame > current_frame {
                self.current_time = self.instant.elapsed();
            }
            
            if frame >= current_frame || current_frame - frame > 100 {
                self.current_frame.store(frame, Ordering::SeqCst);
                return true;
            }
            else {
                return false;
            }
        }
        false
    }

    fn process_osc_message(&mut self, message: OscMessage) -> Result<(), TuioError> {
        match message.addr.as_str() {
            "/tuio/2Dobj" => {
                match message.args.first() {
                    Some(OscType::String(arg)) => {
                        match arg.as_str() {
                            "source" => {
                                if let Some(OscType::String(source_name)) = message.args.get(1) {
                                    self.source_list.entry(source_name.to_string()).or_default();
                                    self.source_name = source_name.to_owned();
                                    Ok(())
                                }
                                else {
                                    Err(TuioError::MissingSourceError(message))
                                }
                            },
                            "alive" => {
                                let to_keep: HashSet<i32> = HashSet::from_iter(message.args.into_iter().skip(1).filter_map(|e| e.int()));
                                let object_map = &mut self.source_list.get_mut(&self.source_name).unwrap().object_map;

                                let mut removed_ids = Vec::with_capacity(object_map.len());

                                object_map.retain(|key, _| {
                                    let keep = to_keep.contains(key);
                                    if !keep {
                                        removed_ids.push(*key);
                                    }
                                    keep
                                });

                                self.dispatcher.remove_objects(&removed_ids);
                                Ok(())
                            },
                            "set" => {
                                if message.args.len() == 11 {
                                    match unwrap_object_args(&message.args) {
                                        Ok(params) => {
                                            self.frame_objects.push(params);
                                            Ok(())
                                        },
                                        Err(index) => Err(TuioError::WrongArgumentTypeError(message, index)),
                                    }
                                }
                                else {
                                    Err(TuioError::MissingArgumentsError(message))
                                }
                            },
                            "fseq" => {
                                if let Some(OscType::Int(fseq)) = message.args.get(1) {
                                    if self.update_frame(*fseq) {
                                        let object_map = &mut self.source_list.get_mut(&self.source_name).unwrap().object_map;

                                        for (session_id, class_id, x_pos, y_pos, angle, x_vel, y_vel, angular_speed, acceleration, angular_acceleration) in self.frame_objects.drain(..) {
                                            match object_map.entry(session_id) {
                                                indexmap::map::Entry::Occupied(mut entry) => {
                                                    let object = entry.get_mut();
                                                    object.update_values(class_id, Point { x: x_pos, y: y_pos }, angle, Velocity{x: x_vel, y: y_vel}, angular_speed, acceleration, angular_acceleration);
                                                    self.dispatcher.update_object(object);
                                                },
                                                indexmap::map::Entry::Vacant(entry) => {
                                                    let object = Object::new(self.current_time, session_id, class_id, Point { x: x_pos, y: y_pos }, angle).with_movement(Velocity{x: x_vel, y: y_vel}, angular_speed, acceleration, angular_acceleration);
                                                    self.dispatcher.add_object(&object);
                                                    entry.insert(object);
                                                },
                                            }
                                        }
                                    }
                                    Ok(())
                                }
                                else {
                                    Err(TuioError::MissingArgumentsError(message))
                                }
                            },
                            _ => Err(TuioError::UnknownMessageTypeError(message))
                        }
                    }
                    None => Err(TuioError::EmptyMessageError(message)),
                    _ => Err(TuioError::UnknownMessageTypeError(message))
                }
            }
            _ => Err(TuioError::EmptyMessageError(message))
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

    fn add_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.dispatcher.add_listener(listener);
    }
    
    fn remove_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.dispatcher.remove_listener(listener);
    }
    
    fn remove_all_listeners(&mut self) {
        self.dispatcher.remove_all_listeners();
    }
}