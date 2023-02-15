use tuio_rs::client::{Client, CursorEvent, BlobEvent, ObjectEvent, TuioEvents};

fn process_events(events: TuioEvents) {
    for event in events.cursor_events {
        match event {
            CursorEvent::New(data) => println!("New Cursor : {:?} from {}", data.cursor, data.source_name),
            CursorEvent::Update(data) => println!("Update Cursor : {:?} from {}", data.cursor, data.source_name),
            CursorEvent::Remove(data) => println!("Remove Cursor : {:?} from {}", data.cursor, data.source_name),
        }
    }

    for event in events.object_events {
        match event {
            ObjectEvent::New(data) => println!("New Object : {:?} from {}", data.object, data.source_name),
            ObjectEvent::Update(data) => println!("Update Object : {:?} from {}", data.object, data.source_name),
            ObjectEvent::Remove(data) => println!("Remove Object : {:?} from {}", data.object, data.source_name),
        }
    }

    for event in events.blob_events {
        match event {
            BlobEvent::New(data) => println!("New Blob : {:?} from {}", data.blob, data.source_name),
            BlobEvent::Update(data) => println!("Update Blob : {:?} from {}", data.blob, data.source_name),
            BlobEvent::Remove(data) => println!("Remove Blob : {:?} from {}", data.blob, data.source_name),
        }
    }
}

fn main() {
    let client = Client::new().unwrap();
    client.connect().expect("Client connecting");

    loop {
        if let Ok(Some(events)) = client.refresh() {
            process_events(events);
        }
    }
}