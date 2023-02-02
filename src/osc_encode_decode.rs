use std::{time::{Duration, SystemTime}, iter};

use rosc::{OscBundle, OscPacket, OscMessage, OscType, OscTime};

use crate::{object::Object, cursor::Cursor, blob::Blob, errors::TuioError};

#[derive(PartialEq, Eq)]
/// Signals the encoding behaviour
pub enum EncodingBehaviour {
    /// Encodes only the TUIO elements updated during the current frame
    CurrentFrame,
    /// Encodes all alive TUIO elements
    Full
}

/// Base trait to implement an OSC encoder
pub trait EncodeOsc<T> {
    /// Encodes an [Object] collection into an OSC bundle
    /// # Arguments
    /// * `object_collection` - an iterable [Object] collection
    /// * `source_name` - the source's name
    /// * `frame_time` - the current's frame time
    /// * `frame_id` - the current's frame id
    /// * `behaviour` - the encoding behaviour
    fn encode_object_bundle<'a, I>(object_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> T where I: IntoIterator<Item = &'a Object>;

    /// Encodes an [Cursor] collection into an OSC bundle
    /// # Arguments
    /// * `cursor_collection` - an iterable [Cursor] collection
    /// * `source_name` - the source's name
    /// * `frame_time` - the current's frame time
    /// * `frame_id` - the current's frame id
    /// * `behaviour` - the encoding behaviour
    fn encode_cursor_bundle<'a, I>(cursor_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> T where I: IntoIterator<Item = &'a Cursor>;

    /// Encodes an [Blob] collection into an OSC bundle
    /// # Arguments
    /// * `blob_collection` - an iterable [Blob] collection
    /// * `source_name` - the source's name
    /// * `frame_time` - the current's frame time
    /// * `frame_id` - the current's frame id
    /// * `behaviour` - the encoding behaviour
    fn encode_blob_bundle<'a, I>(blob_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> T where I: IntoIterator<Item = &'a Blob>;
}

/// An implementation of trait [EncodeOsc] based on [rosc]
pub struct RoscEncoder;

impl EncodeOsc<OscBundle> for RoscEncoder {
    fn encode_object_bundle<'a, I>(object_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> OscBundle where I: IntoIterator<Item = &'a Object> {
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

    fn encode_cursor_bundle<'a, I>(cursor_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> OscBundle where I: IntoIterator<Item = &'a Cursor> {
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

    fn encode_blob_bundle<'a, I>(blob_collection: I, source_name: String, frame_time: Duration, frame_id: i32, behaviour: &EncodingBehaviour) -> OscBundle where I: IntoIterator<Item = &'a Blob> {
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

type ObjectParams = (i32, i32, f32, f32, f32, f32, f32, f32, f32, f32);
type CursorParams = (i32, f32, f32, f32, f32, f32);
type BlobParams = (i32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32);

/// An enum of a "set" TUIO message
pub enum SetParams {
    ObjectParams(Vec<ObjectParams>),
    CursorParams(Vec<CursorParams>),
    BlobParams(Vec<BlobParams>)
}

/// A struct containing informations of a TUIO bundle
pub struct DecodedBundle {
    source: String,
    alive: Vec<i32>,
    set: SetParams,
    fseq: i32
}

/// Base trait to implement an OSC decoder
pub trait DecodeOsc<T> {
    fn decode_bundle(bundle: T) -> Result<DecodedBundle, TuioError>;
}

/// An implementation of trait [DecodeOsc] based on [rosc]
pub struct RoscDecoder;

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

impl DecodeOsc<OscBundle> for RoscDecoder {
    fn decode_bundle(bundle: OscBundle) -> Result<DecodedBundle, TuioError> {
        let mut decoded_bundle: DecodedBundle;
        let message: OscMessage;

        println!("OSC Bundle: {:?}", bundle);
        
        match message.addr.as_str() {
            "/tuio/2Dobj" => decoded_bundle.set = SetParams::ObjectParams(Vec::new()),
            "/tuio/2Dcur" => decoded_bundle.set = SetParams::CursorParams(Vec::new()),
            "/tuio/2Dblb" => decoded_bundle.set = SetParams::BlobParams(Vec::new()),
            _ => return Err(TuioError::EmptyMessageError(message))
        };

        for packet in bundle.content {
            match packet {
                OscPacket::Message(msg) => {
                    println!("OSC address: {}", msg.addr);
                    println!("OSC arguments: {:?}", msg.args);
                    
                    message = msg;
                }
                OscPacket::Bundle(bundle) => {
                    continue;
                }
            }

            match message.args.first() {
                Some(OscType::String(arg)) => {
                    match arg.as_str() {
                        "source" => {
                            decoded_bundle.source = try_unwrap_source_name(message)?;
                        },
                        "alive" => {
                            decoded_bundle.alive = message.args.into_iter().skip(1).filter_map(|e| e.int()).collect();
                        },
                        "set" => {
                            match decoded_bundle.set {
                                SetParams::ObjectParams(set) => {
                                    if message.args.len() != 11 {
                                        return Err(TuioError::MissingArgumentsError(message));
                                    }
                                    
                                    match try_unwrap_object_args(&message.args) {
                                        Ok(params) => {
                                            set.push(params);
                                        },
                                        Err(index) => return Err(TuioError::WrongArgumentTypeError(message, index)),
                                    }
                                },
                                SetParams::CursorParams(set) => {
                                    if message.args.len() != 7 {
                                        return Err(TuioError::MissingArgumentsError(message));
                                    }

                                    match try_unwrap_cursor_args(&message.args) {
                                        Ok(params) => {
                                            set.push(params);
                                        },
                                        Err(index) => return Err(TuioError::WrongArgumentTypeError(message, index)),
                                    }
                                },
                                SetParams::BlobParams(set) => {
                                    if message.args.len() != 13 {
                                        return Err(TuioError::MissingArgumentsError(message));
                                    }

                                    match try_unwrap_blob_args(&message.args) {
                                        Ok(params) => {
                                            set.push(params);
                                        },
                                        Err(index) => return Err(TuioError::WrongArgumentTypeError(message, index)),
                                    }
                                },
                                _ => {return Err(TuioError::EmptyMessageError(message))}
                            };
                        },
                        "fseq" => {
                            if let Some(OscType::Int(fseq)) = message.args.get(1) {
                                decoded_bundle.fseq = *fseq;
                            }
                            else {
                                return Err(TuioError::MissingArgumentsError(message))
                            }
                        },
                        _ => return Err(TuioError::UnknownMessageTypeError(message))
                    }
                },
                None => return Err(TuioError::EmptyMessageError(message)),
                _ => return Err(TuioError::UnknownMessageTypeError(message))
            }
        }

        Ok(decoded_bundle)
    }
}