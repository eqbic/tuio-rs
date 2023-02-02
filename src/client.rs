use std::{time::{Instant, Duration}, net::{SocketAddr, IpAddr, Ipv4Addr, UdpSocket}, thread, sync::{atomic::{AtomicBool, Ordering, AtomicI32}, Arc}, collections::HashSet};

use indexmap::IndexMap;
use rosc::{OscPacket};

use crate::{cursor::{Cursor}, object::Object, blob::Blob, errors::TuioError, listener::{Listener}, dispatcher::{Dispatch, Dispatcher}, osc_encode_decode::{RoscDecoder, DecodeOsc, self, SetParams}};

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

#[derive(Default)]
pub struct SourceCollection {
    pub object_map: IndexMap<i32, Object>,
    pub blob_map: IndexMap<i32, Blob>,
    pub cursor_map: IndexMap<i32, Cursor>
}

pub struct Client<O: OscReceiver> {
    current_frame: AtomicI32,
    instant: Instant,
    current_time: Duration,
    pub source_list: IndexMap<String, SourceCollection>,
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

impl<O: OscReceiver> Client<O>{
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_port(3333)
    }

    pub fn from_port(port: u16) -> Result<Self, std::io::Error> {
        Ok(Self {
            instant: Instant::now(),
            osc_receiver: O::from_port(port)?,
            current_frame: (-1).into(),
            current_time: Duration::default(),
            source_list: IndexMap::new(),
            local_receiver: true,
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
        if frame >= 0 {
            let current_frame = self.current_frame.load(Ordering::SeqCst);
            println!("current_frame {current_frame}");
            if frame > current_frame {
                self.current_time = self.instant.elapsed();
            }
            
            if frame >= current_frame || current_frame - frame > 100 {
                self.current_frame.store(frame, Ordering::SeqCst);
                return true;
            }
            else if self.instant.elapsed() - self.current_time > Duration::from_millis(100){
                self.current_time = self.instant.elapsed();
                return false;
            }
        }
        false
    }

    fn process_osc_packet(&mut self, packet: OscPacket) -> Result<(), TuioError> {
        if let OscPacket::Bundle(bundle) = packet {
            let decoded_bundle = RoscDecoder::decode_bundle(bundle)?;
            
            let to_keep: HashSet<i32> = HashSet::from_iter(decoded_bundle.alive);
            
            if self.update_frame(decoded_bundle.fseq) {
                let source_collection = self.source_list.entry(decoded_bundle.source).or_default();
                match decoded_bundle.tuio_type {
                    osc_encode_decode::TuioBundleType::Cursor => {
                        let cursor_map = &mut source_collection.cursor_map;

                        self.dispatcher.remove_cursors(&retain_by_ids(cursor_map, to_keep));

                        if let Some(SetParams::Cursor(params_collection)) = decoded_bundle.set {
                            for params in params_collection {
                                match cursor_map.entry(params.session_id) {
                                    indexmap::map::Entry::Occupied(mut entry) => {
                                        let cursor = entry.get_mut();
                                        cursor.update_from_params(self.current_time, params);
                                        self.dispatcher.update_cursor(cursor);
                                    },
                                    indexmap::map::Entry::Vacant(entry) => {
                                        let cursor = Cursor::from((self.current_time, params));
                                        self.dispatcher.add_cursor(&cursor);
                                        entry.insert(cursor);
                                    },
                                }
                            }
                        }
                    },
                    osc_encode_decode::TuioBundleType::Object => {
                        let object_map = &mut source_collection.object_map;

                        self.dispatcher.remove_objects(&retain_by_ids(object_map, to_keep));

                        if let Some(SetParams::Object(params_collection)) = decoded_bundle.set {
                            for params in params_collection {
                                match object_map.entry(params.session_id) {
                                    indexmap::map::Entry::Occupied(mut entry) => {
                                        let object = entry.get_mut();
                                        object.update_from_params(self.current_time, params);
                                        self.dispatcher.update_object(object);
                                    },
                                    indexmap::map::Entry::Vacant(entry) => {
                                        let object = Object::from((self.current_time, params));
                                        self.dispatcher.add_object(&object);
                                        entry.insert(object);
                                    },
                                }
                            }
                        }
                    },
                    osc_encode_decode::TuioBundleType::Blob => {
                        let blob_map = &mut source_collection.blob_map;

                        self.dispatcher.remove_blobs(&retain_by_ids(blob_map, to_keep));

                        if let Some(SetParams::Blob(params_collection)) = decoded_bundle.set {
                            for params in params_collection {
                                match blob_map.entry(params.session_id) {
                                    indexmap::map::Entry::Occupied(mut entry) => {
                                        let blob = entry.get_mut();
                                        blob.update_from_params(self.current_time, params);
                                        self.dispatcher.update_blob(blob);
                                    },
                                    indexmap::map::Entry::Vacant(entry) => {
                                        let blob = Blob::from((self.current_time, params));
                                        self.dispatcher.add_blob(&blob);
                                        entry.insert(blob);
                                    },
                                }
                            }
                        }
                    },
                    osc_encode_decode::TuioBundleType::Unknown => (),
                }
            }
            Ok(())
        }
        else {
            Err(TuioError::NotABundle(packet))
        }
    }

    pub fn add_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.dispatcher.add_listener(listener);
    }
    
    pub fn remove_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.dispatcher.remove_listener(listener);
    }
    
    pub fn remove_all_listeners(&mut self) {
        self.dispatcher.remove_all_listeners();
    }

    pub fn local_receiver(&self) -> bool {
        self.local_receiver
    }
}