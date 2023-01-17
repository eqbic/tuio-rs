use std::time::{Duration};

use tuio_rs::{server::Server};

fn main() {
    let mut server = Server::new().unwrap();
    let cursor_id = server.create_cursor(0., 0.);

    // let osc_receiver = OscReceiver::new().unwrap();
    // osc_receiver.listen();

    server.send_full_messages();

    std::thread::sleep(Duration::from_secs(1));
    server.update_cursor(cursor_id, 1., 1.);

    server.send_full_messages();
}