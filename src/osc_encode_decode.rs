use std::{time::{Duration, SystemTime}, iter};

use rosc::{OscBundle, OscPacket, OscMessage, OscType, OscTime};

use crate::{object::Object, cursor::Cursor, blob::Blob};

#[derive(PartialEq, Eq)]
pub enum EncodingBehaviour {
    CurrentFrame,
    Full
}

pub trait EncodeOsc<T> {
    fn encode_object_packet<'a, I>(object_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> T where I: IntoIterator<Item = &'a Object>;
    fn encode_cursor_packet<'a, I>(cursor_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> T where I: IntoIterator<Item = &'a Cursor>;
    fn encode_blob_packet<'a, I>(blob_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> T where I: IntoIterator<Item = &'a Blob>;
}

pub struct RoscEncoder;

impl EncodeOsc<OscPacket> for RoscEncoder {
    fn encode_object_packet<'a, I>(object_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> OscPacket where I: IntoIterator<Item = &'a Object> {
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dobj".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(source_name)
            ]
        });
    
        let mut set_messages = vec![];
        let mut object_ids: Vec<OscType>= vec![];
    
        for object in object_collection.into_iter()  {
            let id = object.get_session_id();
            object_ids.push(OscType::Int(id));
            
            if behaviour == &EncodingBehaviour::CurrentFrame && object.get_time() != frame_time {
                continue;
            }
    
            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dobj".into(),
                args: vec![
                    OscType::String("set".into()),
                    OscType::Int(id),
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
            args: vec![OscType::String("fseq".into()), OscType::Int(frame_id)]
        });
    
        OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(iter::once(frame_message))
            .collect()
        })
    }

    fn encode_cursor_packet<'a, I>(cursor_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> OscPacket where I: IntoIterator<Item = &'a Cursor> {
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(source_name)
            ]
        });
    
        let mut set_messages = vec![];
        let mut cursor_ids: Vec<OscType>= vec![];
    
        for cursor in cursor_collection.into_iter()  {
            let id = cursor.get_session_id();
            cursor_ids.push(OscType::Int(id));
            
            if behaviour == &EncodingBehaviour::CurrentFrame && cursor.get_time() != frame_time {
                continue;
            }

            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dcur".into(),
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
            addr: "/tuio/2Dcur".into(),
            args: vec![OscType::String("alive".into())].into_iter().chain(cursor_ids.into_iter()).collect()
        });
    
        let frame_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dcur".into(),
            args: vec![OscType::String("fseq".into()), OscType::Int(frame_id)]
        });
    
        OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(iter::once(frame_message))
            .collect()
        })
    }

    fn encode_blob_packet<'a, I>(blob_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> OscPacket where I: IntoIterator<Item = &'a Blob> {
        let source_message = OscPacket::Message(OscMessage {
            addr: "/tuio/2Dblb".into(),
            args: vec![
                OscType::String("source".into()),
                OscType::String(source_name)
            ]
        });
    
        let mut set_messages = vec![];
        let mut blob_ids: Vec<OscType>= vec![];
    
        for blob in blob_collection.into_iter() {     
            let id = blob.get_session_id();       
            blob_ids.push(OscType::Int(id));

            if behaviour == &EncodingBehaviour::CurrentFrame && blob.get_time() != frame_time {
                continue;
            }
            
            set_messages.push(OscPacket::Message(OscMessage {
                addr: "/tuio/2Dblb".into(),
                args: vec![
                    OscType::String("set".into()),
                    OscType::Int(id),
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
            args: vec![OscType::String("fseq".into()), OscType::Int(frame_id)]
        });
    
        OscPacket::Bundle(OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(iter::once(frame_message))
            .collect()
        })
    }
}
