use std::{time::{Instant, Duration}, sync::{RwLock, Arc, Mutex}, collections::HashSet, cell::{RefCell, Cell}, thread};

use indexmap::IndexMap;
use ringbuffer::{ConstGenericRingBuffer, RingBufferWrite, RingBufferRead};
use rosc::{OscPacket};

use crate::{osc_receiver::{UdpReceiver, RoscReceiver}, cursor::{Cursor}, object::Object, blob::Blob, errors::{TuioError, OscReceiverError}, osc_encode_decode::{OscDecoder, DecodeOsc, self, Set}};

#[derive(Default)]
pub struct TuioEvents {
    pub cursor_events: Vec<CursorEvent>,
    pub object_events: Vec<ObjectEvent>,
    pub blob_events: Vec<BlobEvent>,
}

pub struct CursorData {
    pub source_name: String,
    pub cursor: Cursor
}

pub struct ObjectData {
    pub source_name: String,
    pub object: Object
}

pub struct BlobData {
    pub source_name: String,
    pub blob: Blob
}

pub enum CursorEvent {
    New(CursorData),
    Update(CursorData),
    Remove(CursorData),
}

pub enum ObjectEvent {
    New(ObjectData),
    Update(ObjectData),
    Remove(ObjectData),
}

pub enum BlobEvent {
    New(BlobData),
    Update(BlobData),
    Remove(BlobData),
}

#[derive(Default)]
pub struct SourceCollection {
    pub object_map: IndexMap<i32, Object>,
    pub blob_map: IndexMap<i32, Blob>,
    pub cursor_map: IndexMap<i32, Cursor>
}

pub struct Client {
    current_frame: Cell<i32>,
    instant: Instant,
    current_time: Cell<Duration>,
    pub source_list: RefCell<IndexMap<String, SourceCollection>>,
    osc_receivers: Vec<Arc<RoscReceiver>>,
    packet_buffer: Arc<Mutex<ConstGenericRingBuffer<OscPacket, 128>>>,
    local_receiver: bool,
    listen: Arc<RwLock<bool>>
}

/// Keeps the entries whose keys are contained in a [HashSet]
/// 
/// Returns a [Vec<i32>] of removed ids
/// 
/// # Arguments
/// * `index_map` - an [IndexMap<i32, T>] to filter
/// * `to_keep` - an [HashSet<i32>] containing the keys to retain
fn retain_by_ids<T>(index_map: &mut IndexMap<i32, T>, to_keep: HashSet<i32>) -> Vec<T> {
    let mut removed: Vec<T> = Vec::with_capacity(index_map.len());
    let mut to_remove: Vec<i32> = Vec::with_capacity(index_map.len());

    for id in index_map.keys() {
        if !to_keep.contains(id) {
            to_remove.push(*id);
        }
    }

    for id in to_remove {
        removed.push(index_map.remove(&id).unwrap());
    }

    removed
}

impl Client {
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_port(3333)
    }

    pub fn from_port(port: u16) -> Result<Self, std::io::Error> {
        Ok(Self {
            instant: Instant::now(),
            osc_receivers: vec![Arc::new(UdpReceiver::from_port(port)?)],
            current_frame: (-1).into(),
            current_time: Cell::new(Duration::default()),
            source_list: RefCell::new(IndexMap::new()),
            local_receiver: true,
            listen: Arc::new(RwLock::new(false)),
            packet_buffer: Default::default()
        })
    }

    pub fn connect(&self) -> Result<(), OscReceiverError> {
        if *self.listen.read().unwrap() {
            return Err(OscReceiverError::AlreadyConnected());
        }

        *self.listen.write().unwrap() = true;

        for receiver in &self.osc_receivers {
            receiver.connect().map_err(OscReceiverError::Connect)?;

            let listen = Arc::clone(&self.listen);
            let receiver = Arc::clone(receiver);
            let buffer = Arc::clone(&self.packet_buffer);

            thread::spawn(move || loop {
                if !*listen.read().unwrap() {
                    break;
                }
                
                match receiver.recv() {
                    Ok(packet) => {
                        buffer.lock().unwrap().push(packet);
                    }
                    Err(err) => {
                        match err {
                            OscReceiverError::Receive(err) => if err.raw_os_error().unwrap() != 10004 {
                                println!("Error receiving from socket: {}", err);
                            },
                            _ => println!("Error receiving from socket: {}", err)
                        }
                        
                        break;
                    }
                }
            });
        }

        Ok(())
    }

    pub fn disconnect(&self) {
        for receiver in &self.osc_receivers {
            receiver.disconnect();
        }
    }

    /// Refreshes the client's state
    /// 
    /// On success, returns an [Option] containing the evnts of all new, updated and removed TUIO inputs
    pub fn refresh(&self) -> Result<Option<TuioEvents>, TuioError> {
        let mut updated = false;
        let mut events = TuioEvents::default();

        for packet in self.packet_buffer.lock().unwrap().drain() {
            if self.process_osc_packet(packet, &mut events)? {
                updated = true;
            }
        };

        if updated {
            Ok(Some(events))
        }
        else {
            Ok(None)
        }
    }

    /// Update frame parameters based on a frame number
    /// 
    /// Returns true if the frame is a new frame
    /// # Argument
    /// * `frame` - the new frame number
    fn update_frame(&self, frame: i32) -> bool {
        if frame >= 0 {
            let current_frame = self.current_frame.get();
            
            if frame > current_frame {
                self.current_time.set(self.instant.elapsed());
            }
            
            if frame >= current_frame || current_frame - frame > 100 {
                self.current_frame.set(frame);
                return true;
            }
            else if self.instant.elapsed() - self.current_time.get() > Duration::from_millis(100){
                self.current_time.set(self.instant.elapsed());
                return false;
            }
        }
        false
    }

    fn process_osc_packet(&self, packet: OscPacket, events: &mut TuioEvents) -> Result<bool, TuioError> {
        if let OscPacket::Bundle(bundle) = packet {
            let decoded_bundle = OscDecoder::decode_bundle(bundle)?;
            
            let to_keep: HashSet<i32> = HashSet::from_iter(decoded_bundle.alive);
            
            if self.update_frame(decoded_bundle.fseq) {
                let mut source_list = self.source_list.borrow_mut();
                let source_collection = source_list.entry(decoded_bundle.source.clone()).or_default(); 
                match decoded_bundle.tuio_type {
                    osc_encode_decode::TuioBundleType::Cursor => {
                        let cursor_map = &mut source_collection.cursor_map;

                        for cursor in retain_by_ids(cursor_map, to_keep).into_iter() {
                            events.cursor_events.push(CursorEvent::Remove(CursorData{source_name: decoded_bundle.source.clone(), cursor: cursor.clone()}));
                        }

                        if let Some(Set::Cursor(cursor_collection)) = decoded_bundle.set {
                            for cursor in cursor_collection {
                                match cursor_map.entry(cursor.get_session_id()) {
                                    indexmap::map::Entry::Occupied(mut entry) => {
                                        events.cursor_events.push(CursorEvent::Update(CursorData{source_name: decoded_bundle.source.clone(), cursor: cursor.clone()}));
                                        entry.insert(cursor);
                                    },
                                    indexmap::map::Entry::Vacant(entry) => {
                                        events.cursor_events.push(CursorEvent::New(CursorData{source_name: decoded_bundle.source.clone(), cursor: cursor.clone()}));
                                        entry.insert(cursor);
                                    },
                                }
                            }
                        }
                    },
                    osc_encode_decode::TuioBundleType::Object => {
                        let object_map = &mut source_collection.object_map;

                        for object in retain_by_ids(object_map, to_keep).into_iter() {
                            events.object_events.push(ObjectEvent::Remove(ObjectData { source_name: decoded_bundle.source.clone(), object: object.clone() }));
                        }

                        if let Some(Set::Object(object_collection)) = decoded_bundle.set {
                            for object in object_collection {
                                match object_map.entry(object.get_session_id()) {
                                    indexmap::map::Entry::Occupied(mut entry) => {
                                        events.object_events.push(ObjectEvent::Update(ObjectData { source_name: decoded_bundle.source.clone(), object: object.clone() }));
                                        entry.insert(object);
                                    },
                                    indexmap::map::Entry::Vacant(entry) => {
                                        events.object_events.push(ObjectEvent::New(ObjectData { source_name: decoded_bundle.source.clone(), object: object.clone() }));
                                        entry.insert(object);
                                    },
                                }
                            }
                        }
                    },
                    osc_encode_decode::TuioBundleType::Blob => {
                        let blob_map = &mut source_collection.blob_map;

                        for blob in retain_by_ids(blob_map, to_keep).into_iter() {
                            events.blob_events.push(BlobEvent::Remove(BlobData { source_name: decoded_bundle.source.clone(), blob: blob.clone() }));
                        }

                        if let Some(Set::Blob(blob_collection)) = decoded_bundle.set {
                            for blob in blob_collection {
                                match blob_map.entry(blob.get_session_id()) {
                                    indexmap::map::Entry::Occupied(mut entry) => {
                                        events.blob_events.push(BlobEvent::Update(BlobData { source_name: decoded_bundle.source.clone(), blob: blob.clone() }));
                                        entry.insert(blob);
                                    },
                                    indexmap::map::Entry::Vacant(entry) => {
                                        events.blob_events.push(BlobEvent::New(BlobData { source_name: decoded_bundle.source.clone(), blob: blob.clone() }));
                                        entry.insert(blob);
                                    },
                                }
                            }
                        }
                    },
                    osc_encode_decode::TuioBundleType::Unknown => (),
                }
                Ok(true)
            }
            else {
                Ok(false)
            }
        }
        else {
            Err(TuioError::NotABundle(packet))
        }
    }

    pub fn local_receiver(&self) -> bool {
        self.local_receiver
    }
}