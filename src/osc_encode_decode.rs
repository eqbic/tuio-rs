use std::{time::{SystemTime}, iter};

use rosc::{OscBundle, OscPacket, OscMessage, OscType, OscTime};

use crate::{object::Object, cursor::{Cursor, Position, Velocity}, blob::Blob, errors::TuioError};

/// Base trait to implement an OSC encoder
pub trait EncodeOsc<T> {
    /// Encodes an [Object] collection into an OSC bundle
    /// # Arguments
    /// * `object_collection` - an iterable [Object] collection
    /// * `source_name` - the source's name
    /// * `frame_id` - the current's frame id
    fn encode_object_bundle<'a, I>(object_collection: I, source_name: String, frame_id: i32) -> T where I: IntoIterator<Item = &'a Object>;

    /// Encodes an [Cursor] collection into an OSC bundle
    /// # Arguments
    /// * `cursor_collection` - an iterable [Cursor] collection
    /// * `source_name` - the source's name
    /// * `frame_id` - the current's frame id
    fn encode_cursor_bundle<'a, I>(cursor_collection: I, source_name: String, frame_id: i32) -> T where I: IntoIterator<Item = &'a Cursor>;

    /// Encodes an [Blob] collection into an OSC bundle
    /// # Arguments
    /// * `blob_collection` - an iterable [Blob] collection
    /// * `source_name` - the source's name
    /// * `frame_id` - the current's frame id
    fn encode_blob_bundle<'a, I>(blob_collection: I, source_name: String, frame_id: i32) -> T where I: IntoIterator<Item = &'a Blob>;
}

/// An implementation of trait [EncodeOsc] based on [rosc]
pub struct OscEncoder;

impl EncodeOsc<OscBundle> for OscEncoder {
    fn encode_object_bundle<'a, I>(object_collection: I, source_name: String, frame_id: i32) -> OscBundle where I: IntoIterator<Item = &'a Object> {
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
                    OscType::Float(object.get_rotation_speed()),
                    OscType::Float(object.get_acceleration()),
                    OscType::Float(object.get_rotation_acceleration())
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
    
        OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(iter::once(frame_message))
            .collect()
        }
    }

    fn encode_cursor_bundle<'a, I>(cursor_collection: I, source_name: String, frame_id: i32) -> OscBundle where I: IntoIterator<Item = &'a Cursor> {
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
    
        OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(iter::once(frame_message))
            .collect()
        }
    }

    fn encode_blob_bundle<'a, I>(blob_collection: I, source_name: String, frame_id: i32) -> OscBundle where I: IntoIterator<Item = &'a Blob> {
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
                    OscType::Float(blob.get_rotation_speed()),
                    OscType::Float(blob.get_acceleration()),
                    OscType::Float(blob.get_rotation_acceleration())
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
    
        OscBundle { 
            timetag: OscTime::try_from(SystemTime::now()).expect("failed with system time conversion"), 
            content: vec![
                source_message,
                alive_message
            ].into_iter()
            .chain(set_messages.into_iter())
            .chain(iter::once(frame_message))
            .collect()
        }
    }
}

/// An enum of a "set" TUIO message
pub enum Set {
    Cursor(Vec<Cursor>),
    Object(Vec<Object>),
    Blob(Vec<Blob>),
}

#[derive(Default)]
pub enum TuioBundleType {
    Cursor,
    Object,
    Blob,
    #[default]
    Unknown
}

/// A struct containing informations of a TUIO bundle
#[derive(Default)]
pub struct TuioBundle {
    pub tuio_type: TuioBundleType,
    pub source: String,
    pub alive: Vec<i32>,
    pub set: Option<Set>,
    pub fseq: i32
}

/// Base trait to implement an OSC decoder
pub trait DecodeOsc<T> {
    fn decode_bundle(bundle: T) -> Result<TuioBundle, TuioError>;
}

/// An implementation of trait [DecodeOsc] based on [rosc]
pub struct OscDecoder;

fn try_unwrap_source_name(message: &OscMessage) -> Result<String, TuioError> {
    match message.args.get(1) {
        Some(arg) => {
            match arg.clone().string() {
                Some(source_name) => Ok(source_name),
                None => Err(TuioError::WrongArgumentType(message.clone(), 1)),
            }
        },
        None => Err(TuioError::MissingSource(message.clone())),
    }
}

fn try_unwrap_object_args(args: &[OscType]) -> Result<Object, u8> {
    Ok(Object {
        session_id: args[1].clone().int().ok_or(1)?,
        class_id: args[2].clone().int().ok_or(2)?,
        position: Position {x: args[3].clone().float().ok_or(3)?, y: args[4].clone().float().ok_or(4)?},
        angle: args[5].clone().float().ok_or(5)?,
        velocity: Velocity {x: args[6].clone().float().ok_or(6)?, y: args[7].clone().float().ok_or(7)?},
        rotation_speed: args[8].clone().float().ok_or(8)?,
        acceleration: args[9].clone().float().ok_or(9)?,
        rotation_acceleration: args[10].clone().float().ok_or(10)?,
    })
}

fn try_unwrap_cursor_args(args: &[OscType]) -> Result<Cursor, u8> {
    Ok(Cursor {
        session_id: args[1].clone().int().ok_or(1)?,
        position: Position {x: args[2].clone().float().ok_or(2)?, y: args[3].clone().float().ok_or(3)?},
        velocity: Velocity {x: args[4].clone().float().ok_or(4)?, y: args[5].clone().float().ok_or(5)?},
        acceleration: args[6].clone().float().ok_or(6)?,
    })
}

fn try_unwrap_blob_args(args: &[OscType]) -> Result<Blob, u8> {
    Ok(Blob {
        session_id: args[1].clone().int().ok_or(1)?,
        position: Position {x: args[2].clone().float().ok_or(2)?, y: args[3].clone().float().ok_or(3)?},
        angle: args[4].clone().float().ok_or(4)?,
        width: args[5].clone().float().ok_or(5)?,
        height: args[6].clone().float().ok_or(6)?,
        area: args[7].clone().float().ok_or(7)?,
        velocity: Velocity {x: args[8].clone().float().ok_or(8)?, y: args[9].clone().float().ok_or(9)?},
        rotation_speed: args[10].clone().float().ok_or(10)?,
        acceleration: args[11].clone().float().ok_or(11)?,
        rotation_acceleration: args[12].clone().float().ok_or(12)?,
    })
}

impl DecodeOsc<OscBundle> for OscDecoder {
    fn decode_bundle(bundle: OscBundle) -> Result<TuioBundle, TuioError> {
        let mut decoded_bundle = TuioBundle::default();
        
        for packet in &bundle.content {
            if let OscPacket::Message(message) = packet {
                match message.args.first() {
                    Some(OscType::String(arg)) => {
                        match arg.as_str() {
                            "source" => {
                                decoded_bundle.tuio_type = match message.addr.as_str() {
                                    "/tuio/2Dobj" => TuioBundleType::Object,
                                    "/tuio/2Dcur" => TuioBundleType::Cursor,
                                    "/tuio/2Dblb" => TuioBundleType::Blob,
                                    _ => return Err(TuioError::UnknownAddress(message.clone()))
                                };

                                decoded_bundle.source = try_unwrap_source_name(message)?;
                            },
                            "alive" => {
                                decoded_bundle.alive = message.args.iter().skip(1).filter_map(|e| e.clone().int()).collect();
                            },
                            "set" => {
                                match decoded_bundle.tuio_type {
                                    TuioBundleType::Cursor => {
                                        if let Set::Cursor(set) = decoded_bundle.set.get_or_insert(Set::Cursor(Vec::new())) {
                                            if message.args.len() != 7 {
                                                return Err(TuioError::MissingArguments(message.clone()));
                                            }
        
                                            match try_unwrap_cursor_args(&message.args) {
                                                Ok(params) => {
                                                    set.push(params);
                                                },
                                                Err(index) => return Err(TuioError::WrongArgumentType(message.clone(), index)),
                                            }
                                        }
                                    },
                                    TuioBundleType::Object => {
                                        if let Set::Object(set) = decoded_bundle.set.get_or_insert(Set::Object(Vec::new())) {
                                            if message.args.len() != 11 {
                                                return Err(TuioError::MissingArguments(message.clone()));
                                            }
                                            
                                            match try_unwrap_object_args(&message.args) {
                                                Ok(params) => {
                                                    set.push(params);
                                                },
                                                Err(index) => return Err(TuioError::WrongArgumentType(message.clone(), index)),
                                            }
                                        }
                                    },
                                    TuioBundleType::Blob => {
                                        if let Set::Blob(set) = decoded_bundle.set.get_or_insert(Set::Blob(Vec::new())) {
                                            if message.args.len() != 13 {
                                                return Err(TuioError::MissingArguments(message.clone()));
                                            }
        
                                            match try_unwrap_blob_args(&message.args) {
                                                Ok(params) => {
                                                    set.push(params);
                                                },
                                                Err(index) => return Err(TuioError::WrongArgumentType(message.clone(), index)),
                                            }
                                        }

                                    },
                                    TuioBundleType::Unknown => return Err(TuioError::IncompleteBundle(bundle)),
                                }
                            },
                            "fseq" => {
                                if let Some(OscType::Int(fseq)) = message.args.get(1) {
                                    decoded_bundle.fseq = *fseq;
                                }
                                else {
                                    return Err(TuioError::MissingArguments(message.clone()))
                                }
                            },
                            _ => return Err(TuioError::UnknownMessageType(message.clone()))
                        }
                    },
                    None => return Err(TuioError::EmptyMessage(message.clone())),
                    _ => return Err(TuioError::UnknownMessageType(message.clone()))
                }
            };
        }

        Ok(decoded_bundle)
    }
}

#[cfg(test)]
mod tests {
    use crate::{cursor::{Cursor, Position}, object::Object, blob::Blob, osc_encode_decode::{OscEncoder, EncodeOsc}};

    use super::*;

    #[test]
    fn encoding_decoding() {
        let source = "test".to_string();

        let cursors = vec![Cursor::new(0, Position {x: 0., y: 0.}), Cursor::new(1, Position {x: 0.5, y: 0.5})];
        let objects = vec![Object::new(0, 0, Position {x: 0., y: 0.}, 0.), Object::new(1, 1, Position {x: 0.5, y: 0.5}, 0.)];
        let blobs = vec![Blob::new(0, Position {x: 0., y: 0.}, 0., 0.3, 0.3, 0.09), Blob::new(1, Position {x: 0.5, y: 0.5}, 0., 0.5, 0.5, 0.25)];

        let cursor_bundle = OscEncoder::encode_cursor_bundle(&cursors, source.clone(), 0);
        let object_bundle = OscEncoder::encode_object_bundle(&objects, source.clone(), 0);
        let blob_bundle = OscEncoder::encode_blob_bundle(&blobs, source, 0);

        match OscDecoder::decode_bundle(cursor_bundle) {
            Ok(decoded_bundle) => {
                if let Some(Set::Cursor(decoded_cursors)) = decoded_bundle.set {
                    assert_eq!(decoded_cursors.len(), 2);
                    assert_eq!(cursors[0], decoded_cursors[0]);
                    assert_eq!(cursors[1], decoded_cursors[1]);
                }
            },
            Err(err) => {println!("{err}"); panic!()},
        }

        match OscDecoder::decode_bundle(object_bundle) {
            Ok(decoded_bundle) => {
                if let Some(Set::Object(decoded_objects)) = decoded_bundle.set {
                    assert_eq!(decoded_objects.len(), 2);
                    assert_eq!(objects[0], decoded_objects[0]);
                    assert_eq!(objects[1], decoded_objects[1]);
                }
            },
            Err(err) => {println!("{err}"); panic!()},
        }

        match OscDecoder::decode_bundle(blob_bundle) {
            Ok(decoded_bundle) => {
                if let Some(Set::Blob(decoded_blobs)) = decoded_bundle.set {
                    assert_eq!(decoded_blobs.len(), 2);
                    assert_eq!(blobs[0], decoded_blobs[0]);
                    assert_eq!(blobs[1], decoded_blobs[1]);
                }
            },
            Err(err) => {println!("{err}"); panic!()},
        }
    }
}
