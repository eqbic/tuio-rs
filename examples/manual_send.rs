use std::{net::{SocketAddr, IpAddr, Ipv4Addr}};

use rosc::OscPacket;
use tuio_rs::{Cursor, cursor::Position, osc_encode_decode::{OscEncoder, EncodeOsc}, server::{UdpSender, SendOsc}};

fn main() {
    let source = "test".to_string();

    let sender = UdpSender::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3333)).unwrap();

    let cursors = vec![
        Cursor::new(0, Position { x: 0., y: 0. }),
        Cursor::new(1, Position { x: 0.5, y: 0.5 }),
    ];

    let cursor_bundle = OscEncoder::encode_cursor_bundle(
        &cursors,
        source,
        0
    );

    sender.send_osc_packet(&OscPacket::Bundle(cursor_bundle)).expect("Sending OSC packet");
}
