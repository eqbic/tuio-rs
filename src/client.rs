use std::{time::{Instant, Duration}, net::{SocketAddr, IpAddr, Ipv4Addr, UdpSocket}, thread, sync::{atomic::{AtomicBool, Ordering, AtomicI32}, Arc}, collections::HashSet, fmt::Error, borrow::Borrow};

use indexmap::IndexMap;
use rosc::{OscPacket, OscMessage, OscType, OscBundle};

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
type CursorParams = (i32, f32, f32, f32, f32, f32);
type BlobParams = (i32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32);

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
    frame_cursors: Vec<CursorParams>,
    frame_blobs: Vec<BlobParams>,
    source_list: IndexMap<String, SourceCollection>,
    source_id: u32,
    source_name: String,
    source_address: SocketAddr,
    osc_receiver: O,
    dispatcher: Dispatcher,
    local_receiver: bool
}

/// Keeps the entries whose keys are contained in a [HashSet]
/// 
/// Returns a [Vec<i32>] of removed ids
/// 
/// # Arguments
/// * `index_map` - an [IndexMap<i32, T>] to filter
/// * `to_keep` - an [HashSet<i32>] containing the keys to retain
fn retain_by_ids<T>(index_map: &mut IndexMap<i32, T>, to_keep: HashSet<i32>) -> Vec<i32> {
    let mut removed_ids = Vec::with_capacity(index_map.len());

    index_map.retain(|key, _| {
        let keep = to_keep.contains(key);
        if !keep {
            removed_ids.push(*key);
        }
        keep
    });

    removed_ids
}

fn try_unwrap_source_name(message: OscMessage) -> Result<String, TuioError> {
    match message.args.get(1) {
        Some(arg) => {
            match arg.clone().string() {
                Some(source_name) => Ok(source_name),
                None => Err(TuioError::WrongArgumentTypeError(message, 1)),
            }
        },
        None => Err(TuioError::MissingSourceError(message)),
    }
}

fn try_unwrap_object_args(args: &[OscType]) -> Result<ObjectParams, u8> {
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

fn try_unwrap_cursor_args(args: &[OscType]) -> Result<CursorParams, u8> {
    Ok((
        args[1].clone().int().ok_or(1)?,
        args[2].clone().float().ok_or(2)?,
        args[3].clone().float().ok_or(3)?,
        args[4].clone().float().ok_or(4)?,
        args[5].clone().float().ok_or(5)?,
        args[6].clone().float().ok_or(6)?,
    ))
}

fn try_unwrap_blob_args(args: &[OscType]) -> Result<BlobParams, u8> {
    Ok((
        args[1].clone().int().ok_or(1)?,
        args[2].clone().float().ok_or(2)?,
        args[3].clone().float().ok_or(3)?,
        args[4].clone().float().ok_or(4)?,
        args[5].clone().float().ok_or(5)?,
        args[6].clone().float().ok_or(6)?,
        args[7].clone().float().ok_or(7)?,
        args[8].clone().float().ok_or(8)?,
        args[9].clone().float().ok_or(9)?,
        args[10].clone().float().ok_or(10)?,
        args[11].clone().float().ok_or(11)?,
        args[12].clone().float().ok_or(12)?,
    ))
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
            frame_cursors: Vec::new(),
            frame_blobs: Vec::new(),
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

    fn set_source_name(&mut self, source_name: String) {
        self.source_list.entry(source_name.to_string()).or_default();
        self.source_name = source_name.to_owned();
    }

    fn process_osc_message(&mut self, message: OscMessage) -> Result<(), TuioError> {
        match message.addr.as_str() {
            "/tuio/2Dobj" => {
                match message.args.first() {
                    Some(OscType::String(arg)) => {
                        match arg.as_str() {
                            "source" => {
                                self.set_source_name(try_unwrap_source_name(message)?);
                                Ok(())
                            },
                            "alive" => {
                                let to_keep: HashSet<i32> = HashSet::from_iter(message.args.into_iter().skip(1).filter_map(|e| e.int()));
                                let object_map = &mut self.source_list.get_mut(&self.source_name).unwrap().object_map;
                                self.dispatcher.remove_objects(&retain_by_ids(object_map, to_keep));
                                Ok(())
                            },
                            "set" => {
                                if message.args.len() == 11 {
                                    match try_unwrap_object_args(&message.args) {
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
            },
            "/tuio/2Dcur" => {
                match message.args.first() {
                    Some(OscType::String(arg)) => {
                        match arg.as_str() {
                            "source" => {
                                self.set_source_name(try_unwrap_source_name(message)?);
                                Ok(())
                            },
                            "alive" => {
                                let to_keep: HashSet<i32> = HashSet::from_iter(message.args.into_iter().skip(1).filter_map(|e| e.int()));
                                let cursor_map = &mut self.source_list.get_mut(&self.source_name).unwrap().cursor_map;
                                self.dispatcher.remove_objects(&retain_by_ids(cursor_map, to_keep));
                                Ok(())
                            },
                            "set" => {
                                if message.args.len() == 11 {
                                    match try_unwrap_cursor_args(&message.args) {
                                        Ok(params) => {
                                            self.frame_cursors.push(params);
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
                                        let cursor_map = &mut self.source_list.get_mut(&self.source_name).unwrap().cursor_map;

                                        for (session_id, x_pos, y_pos, x_vel, y_vel, acceleration) in self.frame_cursors.drain(..) {
                                            match cursor_map.entry(session_id) {
                                                indexmap::map::Entry::Occupied(mut entry) => {
                                                    let cursor = entry.get_mut();
                                                    cursor.update_values(Point { x: x_pos, y: y_pos }, Velocity{x: x_vel, y: y_vel}, acceleration);
                                                    self.dispatcher.update_cursor(cursor);
                                                },
                                                indexmap::map::Entry::Vacant(entry) => {
                                                    let cursor = Cursor::new(self.current_time, session_id, Point { x: x_pos, y: y_pos }).with_movement(Velocity{x: x_vel, y: y_vel}, acceleration);
                                                    self.dispatcher.add_cursor(&cursor);
                                                    entry.insert(cursor);
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
            },
            "/tuio/2Dblb" => {
                match message.args.first() {
                    Some(OscType::String(arg)) => {
                        match arg.as_str() {
                            "source" => {
                                self.set_source_name(try_unwrap_source_name(message)?);
                                Ok(())
                            },
                            "alive" => {
                                let to_keep: HashSet<i32> = HashSet::from_iter(message.args.into_iter().skip(1).filter_map(|e| e.int()));
                                let blob_map = &mut self.source_list.get_mut(&self.source_name).unwrap().blob_map;
                                self.dispatcher.remove_objects(&retain_by_ids(blob_map, to_keep));
                                Ok(())
                            },
                            "set" => {
                                if message.args.len() == 11 {
                                    match try_unwrap_blob_args(&message.args) {
                                        Ok(params) => {
                                            self.frame_blobs.push(params);
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
                                        let blob_map = &mut self.source_list.get_mut(&self.source_name).unwrap().blob_map;

                                        for (session_id, x_pos, y_pos, angle, width, height, area, x_vel, y_vel, angular_speed, acceleration, angular_acceleration) in self.frame_blobs.drain(..) {
                                            match blob_map.entry(session_id) {
                                                indexmap::map::Entry::Occupied(mut entry) => {
                                                    let blob = entry.get_mut();
                                                    blob.update_values(Point { x: x_pos, y: y_pos }, angle, width, height, area, Velocity{x: x_vel, y: y_vel}, angular_speed, acceleration, angular_acceleration);
                                                    self.dispatcher.update_blob(blob);
                                                },
                                                indexmap::map::Entry::Vacant(entry) => {
                                                    let blob = Blob::new(self.current_time, session_id, Point { x: x_pos, y: y_pos }, angle, width, height, area).with_movement(Velocity{x: x_vel, y: y_vel}, angular_speed, acceleration, angular_acceleration);
                                                    self.dispatcher.add_blob(&blob);
                                                    entry.insert(blob);
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
            },
            _ => Err(TuioError::EmptyMessageError(message))
        }
    }

    fn process_osc_packet(&mut self, packet: OscPacket) -> Result<(), TuioError> {
        match packet {
            OscPacket::Message(msg) => {
                println!("OSC address: {}", msg.addr);
                println!("OSC arguments: {:?}", msg.args);
                
                self.process_osc_message(msg)
            }
            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);

                for message in bundle.content {
                    self.process_osc_packet(message)?
                }

                Ok(())
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