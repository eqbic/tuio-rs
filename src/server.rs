use std::{net::{SocketAddr, UdpSocket, Ipv4Addr, Ipv6Addr, IpAddr}, sync::atomic::AtomicI32, time::SystemTime};
use rosc::{OscPacket, OscMessage, OscBundle, OscTime, OscError};
use rosc::encoder;
use rosc::OscType;

use crate::cursor::Cursor; 

struct OscSender {
    socket: UdpSocket,
    address: SocketAddr
}

impl OscSender {
    pub fn new(host: SocketAddr) -> Result<Self, std::io::Error> {
        let ip_address: IpAddr = if host.is_ipv4() {IpAddr::V4(Ipv4Addr::LOCALHOST)} else {IpAddr::V6(Ipv6Addr::LOCALHOST)};
        Ok(Self {socket: UdpSocket::bind(SocketAddr::new(ip_address, 3333))?, address: host})
    }

    pub fn send_osc_packet(&self, packet: &OscPacket) -> Result<(), OscError>{
        let buffer = encoder::encode(packet)?;
        self.socket.send_to(&buffer, self.address).unwrap();
        Ok(())
    }
}

struct Server {
    sender_list: Vec<OscSender>,
    source_name: String,
    cursor_list: Vec<Cursor>,
    last_frame_id: AtomicI32
}

impl Server {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            sender_list: vec![OscSender::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333))?],
            source_name: String::new(),
            cursor_list: Vec::new(),
            last_frame_id: AtomicI32::new(0)
        })
    }
    
    pub fn from_socket_addr(socket_addr: SocketAddr) -> Result<Self, std::io::Error> {
        Ok(Self {
            sender_list: vec![OscSender::new(socket_addr)?],
            source_name: String::new(),
            cursor_list: Vec::new(),
            last_frame_id: AtomicI32::new(0)
        })
    }

    pub fn from_osc_sender(osc_sender: OscSender) -> Self {
        Self {
            sender_list: vec![osc_sender],
            source_name: String::new(),
            cursor_list: Vec::new(),
            last_frame_id: AtomicI32::new(0)
        }
    }

    pub fn add_osc_sender(&mut self, osc_sender: OscSender) {
        self.sender_list.push(osc_sender);
    }

    pub fn set_source_name(&mut self, name: String) {
        self.source_name = name;
    }

    pub fn send_full_messages(&self) {
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2DCur".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(self.source_name.clone())
            ]
        });

        let mut set_messages = vec![];
        let mut cursor_ids: Vec<OscType>= vec![];

        for cursor in &self.cursor_list  {
            let id = cursor.get_id() as i32;
            cursor_ids.push(OscType::Int(id));

            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2DCur".into(),
                args: vec![
                    OscType::String("set".into()),
                    OscType::Int(id),
                    OscType::Float(cursor.get_x_position()),
                    OscType::Float(cursor.get_y_position()),
                    OscType::Float(cursor.get_x_velocity()),
                    OscType::Float(cursor.get_y_velocity()),
                    OscType::Float(cursor.get_acceleration())
                ]
            }));
        }
        
        let alive_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2DCur".into(),
            args: vec![OscType::String("alive".into())].into_iter().chain(cursor_ids.into_iter()).collect()
        });

        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2DCur".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(self.last_frame_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))]
        });

        let bundle = OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(vec![frame_message].into_iter())
            .collect()
        });

        self.deliver_osc_packet(bundle);
    }

    fn deliver_osc_packet(&self, packet: OscPacket) {
        for sender in &self.sender_list {
            sender.send_osc_packet(&packet).expect("invalid packet")
        }
    }
}