pub mod cursor;
pub mod server;
pub mod client;
mod listener;
mod dispatcher;
mod object;
mod blob;
mod errors;
mod osc_encode_decode;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{cursor::{Cursor, Point}, object::Object, blob::Blob, osc_encode_decode::{RoscEncoder, EncodeOsc}, client::{Client, UdpReceiver}};

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn encoding_decoding() {
        let frame_time = Duration::default();
        let source = "test".to_string();

        let cursors = vec![Cursor::new(frame_time, 0, Point {x: 0., y: 0.}), Cursor::new(Duration::from_secs(0), 1, Point {x: 0.5, y: 0.5})];
        let objects = vec![Object::new(frame_time, 0, 0, Point {x: 0., y: 0.}, 0.), Object::new(Duration::from_secs(0), 1, 1, Point {x: 0.5, y: 0.5}, 0.)];
        let blobs = vec![Blob::new(frame_time, 0, Point {x: 0., y: 0.}, 0., 0.3, 0.3, 0.09), Blob::new(Duration::from_secs(0), 1, Point {x: 0.5, y: 0.5}, 0., 0.5, 0.5, 0.25)];

        let cursor_packet = RoscEncoder::encode_cursor_packet(&cursors, source.clone(), frame_time, 0, &osc_encode_decode::EncodingBehaviour::CurrentFrame);
        let object_packet = RoscEncoder::encode_object_packet(&objects, source.clone(), frame_time, 0, &osc_encode_decode::EncodingBehaviour::CurrentFrame);
        let blob_packet = RoscEncoder::encode_blob_packet(&blobs, source.clone(), frame_time, 0, &osc_encode_decode::EncodingBehaviour::CurrentFrame);
    
        let mut client = Client::<UdpReceiver>::new().unwrap();
        
        match client.process_osc_packet(cursor_packet) {
            Ok(_) => {},
            Err(err) => {println!("{err}"); assert!(false)},
        }

        match client.process_osc_packet(object_packet) {
            Ok(_) => {},
            Err(err) => {println!("{err}"); assert!(false)},
        }
        match
         client.process_osc_packet(blob_packet) {
            Ok(_) => {},
            Err(err) => {println!("{err}"); assert!(false)},
        }
        // client.process_osc_packet(object_packet);
        // client.process_osc_packet(blob_packet);

        let collection = client.source_list.get(&source).unwrap();

        assert_eq!(collection.cursor_map.len(), 2);
        assert_eq!(collection.cursor_map[0], cursors[0]);
        assert_eq!(collection.cursor_map[1], cursors[1]);

        assert_eq!(collection.object_map.len(), 2);
        assert_eq!(collection.object_map[0], objects[0]);
        assert_eq!(collection.object_map[1], objects[1]);

        assert_eq!(collection.blob_map.len(), 2);
        assert_eq!(collection.blob_map[0], blobs[0]);
        assert_eq!(collection.blob_map[1], blobs[1]);
    }
}
