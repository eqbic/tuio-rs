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


pub struct ObjectParams {
    pub session_id: i32,
    pub class_id: i32,
    pub x_pos: f32,
    pub y_pos: f32,
    pub angle: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub rotation_speed: f32,
    pub acceleration: f32,
    pub rotation_acceleration: f32
}

pub struct CursorParams {
    pub session_id: i32,
    pub x_pos: f32,
    pub y_pos: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub acceleration: f32
}

pub struct BlobParams {
    pub session_id: i32,
    pub x_pos: f32,
    pub y_pos: f32,
    pub angle: f32,
    pub width: f32,
    pub height: f32,
    pub area: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub rotation_speed: f32,
    pub acceleration: f32,
    pub rotation_acceleration: f32
}

/// An enum of a "set" TUIO message
pub enum SetParams {
    Cursor(Vec<CursorParams>),
    Object(Vec<ObjectParams>),
    Blob(Vec<BlobParams>),
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
    pub set: Option<SetParams>,
    pub fseq: i32
}

/// Base trait to implement an OSC decoder
pub trait DecodeOsc<T> {
    fn decode_bundle(bundle: T) -> Result<TuioBundle, TuioError>;
}

/// An implementation of trait [DecodeOsc] based on [rosc]
pub struct RoscDecoder;

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

fn try_unwrap_object_args(args: &[OscType]) -> Result<ObjectParams, u8> {
    Ok(ObjectParams {
        session_id: args[1].clone().int().ok_or(1)?,
        class_id: args[2].clone().int().ok_or(2)?,
        x_pos: args[3].clone().float().ok_or(3)?,
        y_pos: args[4].clone().float().ok_or(4)?,
        angle: args[5].clone().float().ok_or(5)?,
        x_vel: args[6].clone().float().ok_or(6)?,
        y_vel: args[7].clone().float().ok_or(7)?,
        rotation_speed: args[8].clone().float().ok_or(8)?,
        acceleration: args[9].clone().float().ok_or(9)?,
        rotation_acceleration: args[10].clone().float().ok_or(10)?,
    })
}

fn try_unwrap_cursor_args(args: &[OscType]) -> Result<CursorParams, u8> {
    Ok(CursorParams {
        session_id: args[1].clone().int().ok_or(1)?,
        x_pos: args[2].clone().float().ok_or(2)?,
        y_pos: args[3].clone().float().ok_or(3)?,
        x_vel: args[4].clone().float().ok_or(4)?,
        y_vel: args[5].clone().float().ok_or(5)?,
        acceleration: args[6].clone().float().ok_or(6)?,
    })
}

fn try_unwrap_blob_args(args: &[OscType]) -> Result<BlobParams, u8> {
    Ok(BlobParams {
        session_id: args[1].clone().int().ok_or(1)?,
        x_pos: args[2].clone().float().ok_or(2)?,
        y_pos: args[3].clone().float().ok_or(3)?,
        angle: args[4].clone().float().ok_or(4)?,
        width: args[5].clone().float().ok_or(5)?,
        height: args[6].clone().float().ok_or(6)?,
        area: args[7].clone().float().ok_or(7)?,
        x_vel: args[8].clone().float().ok_or(8)?,
        y_vel: args[9].clone().float().ok_or(9)?,
        rotation_speed: args[10].clone().float().ok_or(10)?,
        acceleration: args[11].clone().float().ok_or(11)?,
        rotation_acceleration: args[12].clone().float().ok_or(12)?,
    })
}

impl DecodeOsc<OscBundle> for RoscDecoder {
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
                                let set = decoded_bundle.set.get_or_insert(
                                    match decoded_bundle.tuio_type {
                                        TuioBundleType::Cursor => SetParams::Cursor(Vec::new()),
                                        TuioBundleType::Object => SetParams::Object(Vec::new()),
                                        TuioBundleType::Blob => SetParams::Blob(Vec::new()),
                                        TuioBundleType::Unknown => return Err(TuioError::IncompleteBundle(bundle)),
                                    }
                                );
    
                                match set {
                                    SetParams::Object(ref mut set) => {
                                        if message.args.len() != 11 {
                                            return Err(TuioError::MissingArguments(message.clone()));
                                        }
                                        
                                        match try_unwrap_object_args(&message.args) {
                                            Ok(params) => {
                                                set.push(params);
                                            },
                                            Err(index) => return Err(TuioError::WrongArgumentType(message.clone(), index)),
                                        }
                                    },
                                    SetParams::Cursor(ref mut set) => {
                                        if message.args.len() != 7 {
                                            return Err(TuioError::MissingArguments(message.clone()));
                                        }
    
                                        match try_unwrap_cursor_args(&message.args) {
                                            Ok(params) => {
                                                set.push(params);
                                            },
                                            Err(index) => return Err(TuioError::WrongArgumentType(message.clone(), index)),
                                        }
                                    },
                                    SetParams::Blob(ref mut set) => {
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
                                };
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
    use std::time::Duration;

    use crate::{cursor::{Cursor, Position}, object::Object, blob::Blob, osc_encode_decode::{RoscEncoder, EncodeOsc, self}};

    use super::*;

    #[test]
    fn encoding_decoding() {
        let frame_time = Duration::default();
        let source = "test".to_string();

        let cursors = vec![Cursor::new(frame_time, 0, Position {x: 0., y: 0.}), Cursor::new(Duration::from_secs(0), 1, Position {x: 0.5, y: 0.5})];
        let objects = vec![Object::new(frame_time, 0, 0, Position {x: 0., y: 0.}, 0.), Object::new(Duration::from_secs(0), 1, 1, Position {x: 0.5, y: 0.5}, 0.)];
        let blobs = vec![Blob::new(frame_time, 0, Position {x: 0., y: 0.}, 0., 0.3, 0.3, 0.09), Blob::new(Duration::from_secs(0), 1, Position {x: 0.5, y: 0.5}, 0., 0.5, 0.5, 0.25)];

        let cursor_bundle = RoscEncoder::encode_cursor_bundle(&cursors, source.clone(), frame_time, 0, &osc_encode_decode::EncodingBehaviour::CurrentFrame);
        let object_bundle = RoscEncoder::encode_object_bundle(&objects, source.clone(), frame_time, 0, &osc_encode_decode::EncodingBehaviour::CurrentFrame);
        let blob_bundle = RoscEncoder::encode_blob_bundle(&blobs, source, frame_time, 0, &osc_encode_decode::EncodingBehaviour::CurrentFrame);
        
        match RoscDecoder::decode_bundle(cursor_bundle) {
            Ok(decoded_bundle) => {
                if let Some(SetParams::Cursor(set)) = decoded_bundle.set {
                    let decoded_cursors: Vec<Cursor> = set.into_iter().map(|params| params.into()).collect();
    
                    assert_eq!(decoded_cursors.len(), 2);
                    assert_eq!(cursors[0], decoded_cursors[0]);
                    assert_eq!(cursors[1], decoded_cursors[1]);
                }
            },
            Err(err) => {println!("{err}"); panic!()},
        }

        match RoscDecoder::decode_bundle(object_bundle) {
            Ok(decoded_bundle) => {
                if let Some(SetParams::Object(set)) = decoded_bundle.set {
                    let decoded_objects: Vec<Object> = set.into_iter().map(|params| params.into()).collect();
    
                    assert_eq!(decoded_objects.len(), 2);
                    assert_eq!(objects[0], decoded_objects[0]);
                    assert_eq!(objects[1], decoded_objects[1]);
                }
            },
            Err(err) => {println!("{err}"); panic!()},
        }

        match RoscDecoder::decode_bundle(blob_bundle) {
            Ok(decoded_bundle) => {
                if let Some(SetParams::Blob(set)) = decoded_bundle.set {
                    let decoded_blobs: Vec<Blob> = set.into_iter().map(|params| params.into()).collect();
    
                    assert_eq!(decoded_blobs.len(), 2);
                    assert_eq!(blobs[0], decoded_blobs[0]);
                    assert_eq!(blobs[1], decoded_blobs[1]);
                }
            },
            Err(err) => {println!("{err}"); panic!()},
        }
    }
}
