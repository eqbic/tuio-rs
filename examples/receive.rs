use tuio_rs::client::{Client, CursorEvent, BlobEvent, ObjectEvent, TuioEvents};

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

    loop {
        if let Ok(Some(events)) = client.refresh() {
            process_events(events);
        }
    }
}