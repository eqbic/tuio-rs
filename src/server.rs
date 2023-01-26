use std::{net::{SocketAddr, UdpSocket, Ipv4Addr, Ipv6Addr, IpAddr}, sync::atomic::{AtomicI32, Ordering}, time::{SystemTime, Instant, Duration}};
use rosc::{OscPacket, OscMessage, OscBundle, OscTime, OscError};
use rosc::encoder;
use rosc::OscType;
use local_ip_address::local_ip;
use indexmap::{IndexMap};

use crate::{cursor::{Cursor, Point}, dispatcher::{Dispatch, Dispatcher}, listener::Listener, object::Object, blob::Blob}; 

/// Base trait to implement sending OSC over various transport methods
pub trait OscSender {
    /// Sends an [OscPacket].
    /// Returns an [OscError] if packet's encoding fails
    ///
    /// # Arguments
    /// * `packet` - a reference to an [OscPacket]
    fn send_osc_packet(&self, packet: &OscPacket) -> Result<(), OscError>;

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

impl OscSender for UdpSender {
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
///- [x] Cursor messaging
///- [ ] Object messaging
///- [ ] Blob messaging
pub struct Server {
    sender_list: Vec<Box<dyn OscSender>>,
    source_name: String,
    dispatcher: Dispatcher,
    session_id: i32,
    object_map: IndexMap<i32, Object>,
    object_updated: bool,
    cursor_map: IndexMap<i32, Cursor>,
    cursor_updated: bool,
    blob_map: IndexMap<i32, Blob>,
    blob_updated: bool,
    instant: Instant,
    current_frame_time: Duration,
    last_frame_id: AtomicI32,
    /// Enables the full update of all currently active and inactive [Object]s, [Cursor]s and [Blob]s
    pub full_update: bool,
    periodic_messaging: bool,
    update_interval: Duration,
    pub object_profiling: bool,
    object_update_time: Duration,
    pub cursor_profiling: bool,
    cursor_update_time: Duration,
    pub blob_profiling: bool,
    blob_update_time: Duration,
}

impl Server {
    /// Creates a TUIO [Server] with a default [UdpSender] configured for 127.0.0.1:3333
    ///
    /// # Arguments
    /// * `packet` - a reference to an [OscPacket]
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333))
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
    pub fn from_osc_sender(osc_sender: impl OscSender + 'static) -> Self {
        Self {
            sender_list: vec![Box::new(osc_sender)],
            source_name: String::new(),
            dispatcher: Dispatcher::new(),
            session_id: -1,
            object_map: IndexMap::new(),
            object_updated: false,
            cursor_map: IndexMap::new(),
            cursor_updated: false,
            blob_map: IndexMap::new(),
            blob_updated: false,
            instant: Instant::now(),
            current_frame_time: Duration::default(),
            last_frame_id: AtomicI32::new(0),
            full_update: false,
            periodic_messaging: false,
            update_interval: Duration::from_secs(1),
            object_profiling: true,
            object_update_time: Duration::default(),
            cursor_profiling: true,
            cursor_update_time: Duration::default(),
            blob_profiling: true,
            blob_update_time: Duration::default(),
        }
    }

    /// Adds an OSC sender implementing [OscSender] trait
    ///
    /// # Arguments
    /// * `osc_sender` - a sender implementing [OscSender]
    pub fn add_osc_sender(&mut self, osc_sender: impl OscSender + 'static) {
        self.sender_list.push(Box::new(osc_sender));
    }

    /// Sets the server source name which is sent through the TUIO source message
    ///
    /// # Arguments
    /// * `name` - the name of the source
    pub fn set_source_name(&mut self, name: String) {
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
        
        let object = Object::new(self.current_frame_time, session_id, class_id, Point{x, y}, angle);
        self.object_map.insert(session_id, object);
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
            object.update(self.current_frame_time, Point{x, y}, angle);
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
        
        let cursor = Cursor::new(self.current_frame_time, session_id, Point{x, y});
        self.cursor_map.insert(session_id, cursor);
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
            cursor.update(self.current_frame_time, Point{x, y});
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
        
        let blob = Blob::new(self.current_frame_time, session_id, Point{x, y}, angle, width, height, area);
        self.blob_map.insert(session_id, blob);
        self.blob_updated = true;
        session_id
    }

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
            blob.update(self.current_frame_time, Point{x, y}, angle, width, height, area);
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
        self.current_frame_time = self.instant.elapsed();
        self.last_frame_id.fetch_add(1, Ordering::SeqCst);
    }

    /// Commits the current frame.
    /// 
    /// Generates and sends TUIO messages of all currently active and updated [Object]s, [Cursor]s and [Blob]s
    pub fn commit_frame(&mut self) {
        let frame_time = self.instant.elapsed();
        for listener in &self.dispatcher.listener_list {
            listener.refresh(frame_time);
        }

        if self.object_updated || (self.periodic_messaging && self.object_profiling && (self.current_frame_time - self.object_update_time) >= self.update_interval) {
            self.deliver_osc_packet(self.build_object_bundle(self.full_update));
            self.object_update_time = self.current_frame_time;
            self.object_updated = false;
        }

        if self.cursor_updated || (self.periodic_messaging && self.cursor_profiling && (self.current_frame_time - self.cursor_update_time) >= self.update_interval) {
            self.deliver_osc_packet(self.build_cursor_bundle(self.full_update));
            self.cursor_update_time = self.current_frame_time;
            self.cursor_updated = false;
        }
        
        if self.blob_updated || (self.periodic_messaging && self.blob_profiling && (self.current_frame_time - self.blob_update_time) >= self.update_interval) {
            self.deliver_osc_packet(self.build_blob_bundle(self.full_update));
            self.blob_update_time = self.current_frame_time;
            self.blob_updated = false;
        }
    }

    fn build_object_bundle(&self, force_all: bool) -> OscPacket{
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });
    
        let mut set_messages = vec![];
        let mut object_ids: Vec<OscType>= vec![];
    
        for (id, object) in &self.object_map  {
            object_ids.push(OscType::Int(*id));
            
            if !force_all && object.get_time() != self.current_frame_time {
                continue;
            }
    
            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dobj".into(),
                args: vec![
                    OscType::String("set".into()),
                    OscType::Int(*id),
                    OscType::Int(object.get_class_id()),
                    OscType::Float(object.get_x_position()),
                    OscType::Float(object.get_y_position()),
                    OscType::Float(object.get_angle()),
                    OscType::Float(object.get_x_velocity()),
                    OscType::Float(object.get_y_velocity()),
                    OscType::Float(object.get_angular_speed()),
                    OscType::Float(object.get_acceleration()),
                    OscType::Float(object.get_angular_acceleration())
                ]
            }));
        }
        
        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![OscType::String("alive".into())].into_iter().chain(object_ids.into_iter()).collect()
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.load(Ordering::SeqCst))]
        });
    
        OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(vec![frame_message].into_iter())
            .collect()
        })
    }

    fn build_cursor_bundle(&self, force_all: bool) -> OscPacket{
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });
    
        let mut set_messages = vec![];
        let mut cursor_ids: Vec<OscType>= vec![];
    
        for (id, cursor) in &self.cursor_map  {
            cursor_ids.push(OscType::Int(*id));
            
            if !force_all && cursor.get_time() != self.current_frame_time {
                continue;
            }

            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dcur".into(),
                args: vec![
                    OscType::String("set".into()),
                    OscType::Int(*id),
                    OscType::Float(cursor.get_x_position()),
                    OscType::Float(cursor.get_y_position()),
                    OscType::Float(cursor.get_x_velocity()),
                    OscType::Float(cursor.get_y_velocity()),
                    OscType::Float(cursor.get_acceleration())
                ]
            }));
        }
        
        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![OscType::String("alive".into())].into_iter().chain(cursor_ids.into_iter()).collect()
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.load(Ordering::SeqCst))]
        });
    
        OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(vec![frame_message].into_iter())
            .collect()
        })
    }

    fn build_blob_bundle(&self, force_all: bool) -> OscPacket{
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });
    
        let mut set_messages = vec![];
        let mut blob_ids: Vec<OscType>= vec![];
    
        for (id, blob) in &self.blob_map {            
            blob_ids.push(OscType::Int(*id));

            if !force_all && blob.get_time() != self.current_frame_time {
                continue;
            }
            
            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dblb".into(),
                args: vec![
                    OscType::String("set".into()),
                    OscType::Int(*id),
                    OscType::Float(blob.get_x_position()),
                    OscType::Float(blob.get_y_position()),
                    OscType::Float(blob.get_angle()),
                    OscType::Float(blob.get_width()),
                    OscType::Float(blob.get_height()),
                    OscType::Float(blob.get_area()),
                    OscType::Float(blob.get_x_velocity()),
                    OscType::Float(blob.get_y_velocity()),
                    OscType::Float(blob.get_angular_speed()),
                    OscType::Float(blob.get_acceleration()),
                    OscType::Float(blob.get_angular_acceleration())
                ]
            }));
        }
        
        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![OscType::String("alive".into())].into_iter().chain(blob_ids.into_iter()).collect()
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.load(Ordering::SeqCst))]
        });
    
        OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(vec![frame_message].into_iter())
            .collect()
        })
    }

    pub fn send_full_messages(&self) {
        self.deliver_osc_packet(self.build_object_bundle(true));
        self.deliver_osc_packet(self.build_cursor_bundle(true));
        self.deliver_osc_packet(self.build_blob_bundle(true));
    }
    
    fn deliver_osc_packet(&self, packet: OscPacket) {
        for sender in &self.sender_list {
            sender.send_osc_packet(&packet).expect("invalid packet")
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