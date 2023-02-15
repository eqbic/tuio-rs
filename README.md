# tuio-rs
A Rust implementation of the [TUIO 1.1 protocol](https://www.tuio.org/?specification) based on [rosc](https://crates.io/crates/rosc)
# Getting Started
## Examples
A simple receiver loop
```sh
cargo run --example receive
```

A server and client communicating together
```sh
cargo run --example send_and_receive
```

Create and send TUIO inputs manually
```sh
cargo run --example manual_send
```
## Create and send TUIO inputs through Server
```rust
use tuio_rs::{Server};

let mut server = Server::new("server_name").unwrap();

// All TUIO inputs manipulation must be made between [Server::init_frame()] and [Server::commit_frame()]
server.init_frame(); 
    let cursor_id = server.create_cursor(0., 0.);
    let object_id = server.create_object(1, 0., 0., 0.);
    let blob_id = server.create_blob(0., 0., 0., 0.1, 0.1, 0.01);
server.commit_frame();

server.init_frame();
    server.update_cursor(cursor_id, 1., 1.);
    server.update_object(object_id, 1., 1., 90f32.to_radians());
    server.update_blob(blob_id, 1., 1., 90f32.to_radians(), 0.2, 0.2, 0.04);
server.commit_frame();

server.init_frame();
    server.remove_cursor(cursor_id);
    server.remove_object(object_id);
    server.remove_blob(blob_id);
server.commit_frame();
```

## Receive TUIO inputs through Client
```rust
use tuio_rs::{Client};

let client = Client::new().unwrap();
client.connect().expect("Client connecting");

loop {
    if let Ok(Some(events)) = client.refresh() {
        // Process events. See receiver example for a process function
    }
}
```