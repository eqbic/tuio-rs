use std::{net::{SocketAddr, UdpSocket, Ipv4Addr, Ipv6Addr, IpAddr}, sync::atomic::{AtomicI32, Ordering}, time::{SystemTime, Instant, Duration}, error::Error};
use rosc::{OscPacket, OscMessage, OscBundle, OscTime, OscError};
use rosc::encoder;
use rosc::OscType;
use local_ip_address::local_ip;
use indexmap::{IndexMap};

use crate::{cursor::{Position}, osc_encode_decode::{EncodeOsc, OscEncoder}, Object, Cursor, Blob}; 

/// Base trait to implement sending OSC over various transport methods
pub trait SendOsc<P, E> where E: Error {
    /// Sends an OSC packet.
    /// Returns an [Error] if packet's encoding fails
    ///
    /// # Arguments
    /// * `packet` - a reference to an OSC packet
    fn send_osc_packet(&self, packet: &P) -> Result<(), E>;

    /// Returns a true if the connection is established
    fn is_connected(&self) -> bool;

    /// Returns true if the target is a loopback address
    fn is_local(&self) -> bool;
}

pub struct UdpSender {
    socket: UdpSocket,
    address: SocketAddr
}

impl UdpSender {
    /// Creates an [UdpSender] binded on localhost
    ///
    /// # Arguments
    /// * `target` - the target socket address
    pub fn new(target: SocketAddr) -> Result<Self, std::io::Error> {
        let ip_address: IpAddr = if target.is_ipv4() {IpAddr::V4(Ipv4Addr::LOCALHOST)} else {IpAddr::V6(Ipv6Addr::LOCALHOST)};
        Ok(Self {socket: UdpSocket::bind(SocketAddr::new(ip_address, 0))?, address: target})
    }
}

impl SendOsc<OscPacket, OscError> for UdpSender {
    /// Sends an [OscPacket] over UDP.
    /// Returns an [OscError] if packet's encoding fails
    ///
    /// # Arguments
    /// * `packet` - a reference to an [OscPacket]
    fn send_osc_packet(&self, packet: &OscPacket) -> Result<(), OscError> {
        let buffer = encoder::encode(packet)?;
        self.socket.send_to(&buffer, self.address).unwrap();
        Ok(())
    }

    /// Always returns true because UDP is connectionless
    fn is_connected(&self) -> bool {
        true
    }

    /// Return true if the target is a loopback address
    fn is_local(&self) -> bool {
        self.address.ip().is_loopback()
    }
}

/// TUIO Server which keeps track of all TUIO elements and which send TUIO messages over the network
pub struct Server {
    sender_list: Vec<Box<dyn SendOsc<OscPacket, OscError>>>,
    source_name: String,
    session_id: i32,
    object_map: IndexMap<i32, Object>,
    object_updated: bool,
    frame_cursor_ids: Vec<i32>,
    frame_object_ids: Vec<i32>,
    frame_blob_ids: Vec<i32>,
    cursor_map: IndexMap<i32, Cursor>,
    cursor_updated: bool,
    blob_map: IndexMap<i32, Blob>,
    blob_updated: bool,
    instant: Instant,
    last_frame_instant: Instant,
    frame_duration: Duration,
    last_frame_id: AtomicI32,
    /// Enables the full update of all currently active and inactive [Object]s, [Cursor]s and [Blob]s
    pub full_update: bool,
    periodic_messaging: bool,
    update_interval: Duration,
    pub object_profiling: bool,
    object_update_time: Instant,
    pub cursor_profiling: bool,
    cursor_update_time: Instant,
    pub blob_profiling: bool,
    blob_update_time: Instant,
}

impl Server {
    /// Creates a TUIO [Server] with a default [UdpSender] configured for 127.0.0.1:3333
    ///
    /// # Arguments
    /// * `source_name` - the source name
    pub fn new(source_name: &str) -> Result<Self, std::io::Error> {
        let mut server = Self::from_socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333))?;
        server.set_source_name(source_name);
        Ok(server)
    }
    
    /// Creates a TUIO [Server] with a [UdpSender] configured from a provided socket address
    ///
    /// # Arguments
    /// * `socket_addr` - a socket address [SocketAddr]
    pub fn from_socket_addr(socket_addr: SocketAddr) -> Result<Self, std::io::Error> {
        Ok(Self::from_osc_sender(UdpSender::new(socket_addr)?))
    }

    /// Creates a TUIO [Server] from an OSC sender implementing [OscSender] trait
    ///
    /// # Arguments
    /// * `osc_sender` - a sender implementing [OscSender]
    pub fn from_osc_sender(osc_sender: impl SendOsc<OscPacket, OscError> + 'static) -> Self {
        Self {
            sender_list: vec![Box::new(osc_sender)],
            source_name: String::new(),
            session_id: -1,
            object_map: IndexMap::new(),
            object_updated: false,
            cursor_map: IndexMap::new(),
            cursor_updated: false,
            blob_map: IndexMap::new(),
            blob_updated: false,
            instant: Instant::now(),
            last_frame_instant: Instant::now(),
            frame_duration: Duration::default(),
            last_frame_id: AtomicI32::new(0),
            full_update: false,
            periodic_messaging: false,
            update_interval: Duration::from_secs(1),
            object_profiling: true,
            object_update_time: Instant::now(),
            cursor_profiling: true,
            cursor_update_time: Instant::now(),
            blob_profiling: true,
            blob_update_time: Instant::now(),
            frame_cursor_ids: Vec::new(),
            frame_object_ids: Vec::new(),
            frame_blob_ids: Vec::new(),
        }
    }

    /// Adds an OSC sender implementing [OscSender] trait
    ///
    /// # Arguments
    /// * `osc_sender` - a sender implementing [OscSender]
    pub fn add_osc_sender(&mut self, osc_sender: impl SendOsc<OscPacket, OscError> + 'static) {
        self.sender_list.push(Box::new(osc_sender));
    }

    /// Sets the server source name which is sent through the TUIO source message
    ///
    /// # Arguments
    /// * `name` - the name of the source
    pub fn set_source_name(&mut self, name: &str) {
        let source = if self.sender_list[0].is_local() {String::from("local")} else {
            match local_ip() {
                Ok(ip) => ip.to_string(),
                Err(_) => String::new()
            }
        };

        self.source_name = format!("{}@{}", name, source);
    }

    /// Enables the periodic full update of all currently active TUIO [Object]s, [Cursor]s and [Blob]s
    ///
    /// # Arguments
    /// * `interval` - an option of a duration. Notes that the minimum interval will always be 10 milliseconds
    pub fn enable_periodic_message(&mut self, interval: Option<Duration>) {
        self.periodic_messaging = true;

        if let Some(new_interval) = interval {
            self.update_interval = new_interval.max(Duration::from_millis(10));
        }
    }

    /// Disable the periodic full update of all currently active TUIO [Object]s, [Cursor]s and [Blob]s
    pub fn disable_periodic_message(&mut self) {
        self.periodic_messaging = false;
    }

    fn get_session_id(&mut self) -> i32 {
        self.session_id += 1;
        self.session_id
    }

    /// Creates a TUIO [Object] and returns its session_id
    ///
    /// # Arguments
    /// * `class_id` - a unique identifier that can be associated with a tangible object
    /// * `x` - the object's x position
    /// * `y` - the object's y position
    /// * `angle` - the object's angle
    pub fn create_object(&mut self, class_id: i32, x: f32, y: f32, angle: f32) -> i32 {
        let session_id = self.get_session_id();
        
        let object = Object::new(session_id, class_id, Position{x, y}, angle);
        self.object_map.insert(session_id, object);
        self.frame_object_ids.push(session_id);
        self.object_updated = true;
        session_id
    }

    /// Updates a TUIO [Object]
    ///
    /// # Arguments
    /// * `session_id` - the object's session id
    /// * `x` - the new object's x position
    /// * `y` - the new object's y position
    /// * `angle` - the new object's angle
    pub fn update_object(&mut self, session_id: i32, x: f32, y: f32, angle: f32) {
        if let Some(object) = self.object_map.get_mut(&session_id) {
            object.update(self.frame_duration, Position{x, y}, angle);
            self.frame_object_ids.push(session_id);
            self.frame_object_ids.push(session_id);
            self.object_updated = true;
        }
    }

    /// Removes a TUIO [Object]
    ///
    /// # Arguments
    /// * `session_id` - the object's session id
    pub fn remove_object(&mut self, session_id: i32) {
        if self.object_map.remove(&session_id).is_some() {
            self.object_updated = true;
        }
    }

    /// Creates a TUIO [Cursor] and returns its session_id
    ///
    /// # Arguments
    /// * `x` - the cursor's x position
    /// * `y` - the cursor's y position
    pub fn create_cursor(&mut self, x: f32, y: f32) -> i32 {
        let session_id = self.get_session_id();
        
        let cursor = Cursor::new(session_id, Position{x, y});
        self.cursor_map.insert(session_id, cursor);
        self.frame_cursor_ids.push(session_id);
        self.cursor_updated = true;
        session_id
    }

    /// Updates a TUIO [Cursor]
    ///
    /// # Arguments
    /// * `session_id` - the cursor's session id
    /// * `x` - the new cursor's x position
    /// * `y` - the new cursor's y position
    pub fn update_cursor(&mut self, session_id: i32, x: f32, y: f32) {
        if let Some(cursor) = self.cursor_map.get_mut(&session_id) {
            cursor.update(self.frame_duration, Position{x, y});
            self.frame_cursor_ids.push(session_id);
            self.cursor_updated = true;
        }
    }

    /// Removes a TUIO [Cursor]
    ///
    /// # Arguments
    /// * `session_id` - the cursor's session id
    pub fn remove_cursor(&mut self, session_id: i32) {
        if self.cursor_map.remove(&session_id).is_some() {
            self.cursor_updated = true;
        }
    }

    /// Creates a TUIO [Blob] and returns its session_id
    ///
    /// # Arguments
    /// * `x` - the blob's x position
    /// * `y` - the blob's y position
    /// * `angle` - the blob's angle
    /// * `width` - the blob's width
    /// * `height` - the blob's height
    /// * `area` - the blob's area
    pub fn create_blob(&mut self, x: f32, y: f32, angle: f32, width: f32, height: f32, area: f32) -> i32 {
        let session_id = self.get_session_id();
        
        let blob = Blob::new(session_id, Position{x, y}, angle, width, height, area);
        self.blob_map.insert(session_id, blob);
        self.frame_blob_ids.push(session_id);
        self.blob_updated = true;
        session_id
    }

    #[allow(clippy::too_many_arguments)]
    /// Updates a TUIO [Blob]
    ///
    /// # Arguments
    /// * `session_id` - the blob's session id
    /// * `x` - the new blob's x position
    /// * `y` - the new blob's y position
    /// * `angle` - the new blob's angle
    /// * `width` - the new blob's width
    /// * `height` - the new blob's height
    /// * `area` - the new blob's area
    pub fn update_blob(&mut self, session_id: i32, x: f32, y: f32, angle: f32, width: f32, height: f32, area: f32) {
        if let Some(blob) = self.blob_map.get_mut(&session_id) {
            blob.update(self.frame_duration, Position{x, y}, angle, width, height, area);
            self.frame_blob_ids.push(session_id);
            self.frame_blob_ids.push(session_id);
            self.blob_updated = true;
        }
    }

    /// Removes a TUIO [Blob]
    ///
    /// # Arguments
    /// * `session_id` - the blob's session id
    pub fn remove_blob(&mut self, session_id: i32) {
        if self.blob_map.remove(&session_id).is_some() {
            self.blob_updated = true;
        }
    }

    /// Initializes a new frame.
    pub fn init_frame(&mut self) {
        self.frame_duration = self.instant.duration_since(self.last_frame_instant);
        self.last_frame_instant = Instant::now();
        self.last_frame_id.fetch_add(1, Ordering::SeqCst);
    }

    /// Commits the current frame.
    /// 
    /// Generates and sends TUIO messages of all currently active and updated [Object]s, [Cursor]s and [Blob]s
    pub fn commit_frame(&mut self) {
        if self.object_updated || (self.periodic_messaging && self.object_profiling && self.object_update_time.duration_since(self.last_frame_instant) >= self.update_interval) {
            if self.full_update {
                let object_collection = self.frame_object_ids.iter().map(|id| self.object_map.get(id).unwrap());
                self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_object_bundle(object_collection, self.source_name.clone(), self.last_frame_id.load(Ordering::SeqCst))));
            }
            else {
                let object_collection = self.object_map.values();
                self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_object_bundle(object_collection, self.source_name.clone(), self.last_frame_id.load(Ordering::SeqCst))));
            }
            
            self.frame_object_ids.clear();
            self.object_update_time = self.last_frame_instant;
            self.object_updated = false;
        }

        if self.cursor_updated || (self.periodic_messaging && self.cursor_profiling && self.cursor_update_time.duration_since(self.last_frame_instant) >= self.update_interval) {
            if !self.full_update {
                let cursor_collection = self.frame_cursor_ids.iter().map(|id| self.cursor_map.get(id).unwrap());
                self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_cursor_bundle(cursor_collection, self.source_name.clone(), self.last_frame_id.load(Ordering::SeqCst))));
            } else {
                let cursor_collection = self.cursor_map.iter().map(|(_, cursor)| cursor);
                self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_cursor_bundle(cursor_collection, self.source_name.clone(), self.last_frame_id.load(Ordering::SeqCst))));
            };

            self.frame_cursor_ids.clear();
            self.cursor_update_time = self.last_frame_instant;
            self.cursor_updated = false;
        }
        
        if self.blob_updated || (self.periodic_messaging && self.blob_profiling && self.blob_update_time.duration_since(self.last_frame_instant) >= self.update_interval) {
            if !self.full_update {
                let blob_collection = self.frame_blob_ids.iter().map(|id| self.blob_map.get(id).unwrap());
                self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_blob_bundle(blob_collection, self.source_name.clone(), self.last_frame_id.load(Ordering::SeqCst))));
            } else {
                let blob_collection = self.blob_map.values();
                self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_blob_bundle(blob_collection, self.source_name.clone(), self.last_frame_id.load(Ordering::SeqCst))));
            };
            
            self.frame_blob_ids.clear();
            self.blob_update_time = self.last_frame_instant;
            self.blob_updated = false;
        }
    }

    pub fn send_full_messages(&self) {
        let frame_id = self.last_frame_id.load(Ordering::SeqCst);
        self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_object_bundle(self.object_map.values(), self.source_name.clone(), frame_id)));
        self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_cursor_bundle(self.cursor_map.values(), self.source_name.clone(), frame_id)));
        self.deliver_osc_packet(OscPacket::Bundle(OscEncoder::encode_blob_bundle(self.blob_map.values(), self.source_name.clone(), frame_id)));
    }
    
    fn deliver_osc_packet(&self, packet: OscPacket) {
        for sender in &self.sender_list {
            sender.send_osc_packet(&packet).expect("invalid packet")
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });

        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![OscType::String("alive".into())]
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(-1)]
        });
    
        let packet = OscPacket::Bundle(OscBundle { 
           timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
           content: vec![
               source_message,
               alive_message,
               frame_message
           ]
        });

        self.deliver_osc_packet(packet);

        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });

        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![OscType::String("alive".into())]
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(-1)]
        });
    
        let packet = OscPacket::Bundle(OscBundle { 
           timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
           content: vec![
               source_message,
               alive_message,
               frame_message
           ]
        });

        self.deliver_osc_packet(packet);

        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });

        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![OscType::String("alive".into())]
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(-1)]
        });
    
        let packet = OscPacket::Bundle(OscBundle { 
           timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
           content: vec![
               source_message,
               alive_message,
               frame_message
           ]
        });

        self.deliver_osc_packet(packet);
    }
}