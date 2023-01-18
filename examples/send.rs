use std::time::{Duration};

use tuio_rs::{server::Server};

fn main() {
    let mut server = Server::new().unwrap();
    // server.enable_periodic_message(None);
    // server.full_update = true;
    let cursor_id = server.create_cursor(0., 0.);
    let object_id = server.create_object(1, 0., 0., 0.);

    // let osc_receiver = OscReceiver::new().unwrap();
    // osc_receiver.listen();

    // server.send_full_messages();
    server.commit_frame();

    std::thread::sleep(Duration::from_secs(2));
    server.init_frame();
    server.update_cursor(cursor_id, 1., 1.);
    
    // server.send_full_messages();
    server.commit_frame();
    std::thread::sleep(Duration::from_secs(2));
}