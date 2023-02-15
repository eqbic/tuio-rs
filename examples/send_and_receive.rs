use std::time::{Duration};

use tuio_rs::{Client, client::{TuioEvents, CursorEvent, ObjectEvent, BlobEvent}, Server};

fn process_events(events: TuioEvents) {
    for event in events.cursor_events {
        match event {
            CursorEvent::New(cursor) => println!("New Cursor : {:?}", cursor),
            CursorEvent::Update(cursor) => println!("Update Cursor : {:?}", cursor),
            CursorEvent::Remove(cursor) => println!("Remove Cursor : {:?}", cursor),
        }
    }

    for event in events.object_events {
        match event {
            ObjectEvent::New(object) => println!("New Object : {:?}", object),
            ObjectEvent::Update(object) => println!("Update Object : {:?}", object),
            ObjectEvent::Remove(object) => println!("Remove Object : {:?}", object),
        }
    }

    for event in events.blob_events {
        match event {
            BlobEvent::New(blob) => println!("New Blob : {:?}", blob),
            BlobEvent::Update(blob) => println!("Update Blob : {:?}", blob),
            BlobEvent::Remove(blob) => println!("Remove Blob : {:?}", blob),
        }
    }
}

fn main() {
    let client = Client::new().unwrap();
    client.connect().expect("Client connecting failed");

    let mut server = Server::new().unwrap();
    
    server.init_frame();
    let cursor_id = server.create_cursor(0., 0.);
    let object_id = server.create_object(1, 0., 0., 0.);
    let blob_id = server.create_blob(0., 0., 0., 0.1, 0.1, 0.01);
    server.commit_frame();
    std::thread::sleep(Duration::from_secs(1));

    if let Ok(Some(events)) = client.refresh() {
        process_events(events);
    }

    server.init_frame();
    server.update_cursor(cursor_id, 1., 1.);
    server.update_object(object_id, 1., 1., 90f32.to_radians());
    server.update_blob(blob_id, 1., 1., 90f32.to_radians(), 0.2, 0.2, 0.04);
    server.commit_frame();

    std::thread::sleep(Duration::from_secs(1));

    if let Ok(Some(events)) = client.refresh() {
        process_events(events);
    }

    server.init_frame();
    server.remove_cursor(cursor_id);
    server.remove_object(object_id);
    server.remove_blob(blob_id);
    server.commit_frame();

    std::thread::sleep(Duration::from_secs(1));

    if let Ok(Some(events)) = client.refresh() {
        process_events(events);
    }
}