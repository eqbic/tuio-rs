use std::{net::{SocketAddr, UdpSocket, Ipv4Addr, Ipv6Addr, IpAddr}, sync::atomic::AtomicI32, time::{SystemTime, Instant}, collections::HashMap};
use rosc::{OscPacket, OscMessage, OscBundle, OscTime, OscError};
use rosc::encoder;
use rosc::OscType;

use crate::{cursor::{Cursor, Point}, dispatcher::Dispatch, listener::Listener, object::Object, blob::Blob}; 

pub trait OscSender {
    fn send_osc_packet(&self, packet: &OscPacket) -> Result<(), OscError>;
    fn is_connected(&self) -> bool;
    fn is_local(&self) -> bool;
}

pub struct UdpSender {
    socket: UdpSocket,
    address: SocketAddr
}

impl UdpSender {
    pub fn new(host: SocketAddr) -> Result<Self, std::io::Error> {
        let ip_address: IpAddr = if host.is_ipv4() {IpAddr::V4(Ipv4Addr::LOCALHOST)} else {IpAddr::V6(Ipv6Addr::LOCALHOST)};
        Ok(Self {socket: UdpSocket::bind(SocketAddr::new(ip_address, 0))?, address: host})
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
    listener_list: Vec<Box<dyn Listener>>,
    object_map: HashMap<i32, Object>,
    free_object_ids: Vec<i32>,
    cursor_map: HashMap<i32, Cursor>,
    free_cursor_ids: Vec<i32>,
    blob_map: HashMap<i32, Blob>,
    free_blob_ids: Vec<i32>,
    last_frame_id: AtomicI32,
    instant: Instant
}

impl Server {
    pub fn new() -> Result<Self, std::io::Error> {
        Self::from_socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333))
    }
    
    pub fn from_socket_addr(socket_addr: SocketAddr) -> Result<Self, std::io::Error> {
        Ok(Self::from_osc_sender(UdpSender::new(socket_addr)?))
    }

    pub fn from_osc_sender(osc_sender: impl OscSender + 'static) -> Self {
        Self {
            sender_list: vec![Box::new(osc_sender)],
            source_name: String::new(),
            listener_list: Vec::new(),
            object_map: HashMap::new(),
            free_object_ids: Vec::new(),
            cursor_map: HashMap::new(),
            free_cursor_ids: Vec::new(),
            blob_map: HashMap::new(),
            free_blob_ids: Vec::new(),
            last_frame_id: AtomicI32::new(0),
            instant: Instant::now()
        }
    }

    pub fn add_osc_sender(&mut self, osc_sender: impl OscSender + 'static) {
        self.sender_list.push(Box::new(osc_sender));
    }

    pub fn set_source_name(&mut self, name: String) {
        self.source_name = name;
    }

    pub fn create_object(&mut self, class_id: i32, x: f32, y: f32, angle: f32) -> i32 {
        let session_id = match self.free_object_ids.pop() {
            Some(id) => id,
            None => self.object_map.len() as i32
        };
        
        let object = Object::new(self.instant, session_id, class_id, Point{x, y}, angle);
        self.object_map.insert(session_id, object);
        session_id
    }

    pub fn update_object(&mut self, session_id: i32, x: f32, y: f32, angle: f32) {
        if let Some(object) = self.object_map.get_mut(&session_id) {
            object.update(Point{x, y}, angle);
        }
    }

    pub fn remove_object(&mut self, session_id: i32) {
        if self.object_map.remove(&session_id).is_some() {
            self.free_object_ids.push(session_id);
        }
    }

    pub fn create_cursor(&mut self, x: f32, y: f32) -> i32 {
        let session_id = match self.free_cursor_ids.pop() {
            Some(id) => id,
            None => self.cursor_map.len() as i32
        };
        
        let cursor = Cursor::new(self.instant, session_id, Point{x, y});
        self.cursor_map.insert(session_id, cursor);
        session_id
    }

    pub fn update_cursor(&mut self, session_id: i32, x: f32, y: f32) {
        if let Some(cursor) = self.cursor_map.get_mut(&session_id) {
            cursor.update(Point{x, y});
        }
    }

    pub fn remove_cursor(&mut self, session_id: i32) {
        if self.cursor_map.remove(&session_id).is_some() {
            self.free_cursor_ids.push(session_id);
        }
    }

    pub fn create_blob(&mut self, x: f32, y: f32, angle: f32, width: f32, height: f32, area: f32) -> i32 {
        let session_id = match self.free_blob_ids.pop() {
            Some(id) => id,
            None => self.blob_map.len() as i32
        };
        
        let blob = Blob::new(self.instant, session_id, Point{x, y}, angle, width, height, area);
        self.blob_map.insert(session_id, blob);
        session_id
    }

    pub fn update_blob(&mut self, session_id: i32, x: f32, y: f32, angle: f32, width: f32, height: f32, area: f32) {
        if let Some(blob) = self.blob_map.get_mut(&session_id) {
            blob.update(Point{x, y}, angle, width, height, area);
        }
    }

    pub fn remove_blob(&mut self, session_id: i32) {
        if self.blob_map.remove(&session_id).is_some() {
            self.free_blob_ids.push(session_id);
        }
    }

    pub fn commit_frame() {
        todo!()
    }
    fn build_object_bundle(&self) -> OscPacket{
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
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))]
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

    fn build_cursor_bundle(&self) -> OscPacket{
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });
    
        let mut set_messages = vec![];
        let mut cursor_ids: Vec<OscType>= vec![];
    
        for (id, cursor) in &self.cursor_map  {
            cursor_ids.push(OscType::Int(*id));
    
            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dobj".into(),
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
            addr: "/tuio/2Dobj".into(),
            args: vec![OscType::String("alive".into())].into_iter().chain(cursor_ids.into_iter()).collect()
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))]
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

    fn build_blob_bundle(&self) -> OscPacket{
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });
    
        let mut set_messages = vec![];
        let mut blob_ids: Vec<OscType>= vec![];
    
        for (id, blob) in &self.blob_map  {
            blob_ids.push(OscType::Int(*id));
    
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
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))]
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
        self.deliver_osc_packet(self.build_object_bundle());
        self.deliver_osc_packet(self.build_cursor_bundle());
        self.deliver_osc_packet(self.build_blob_bundle());
    }
    
    fn deliver_osc_packet(&self, packet: OscPacket) {
        for sender in &self.sender_list {
            sender.send_osc_packet(&packet).expect("invalid packet")
        }
    }
}

impl Dispatch for Server {
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
}