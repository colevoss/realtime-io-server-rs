use std::time::Duration;

use io_server::ioserver::IoServerController;

fn main() {
    let server_controller = IoServerController::new();
    let mut client = server_controller.open_stream("sounds/sample-1.wav".to_string());

    let t = server_controller.start();

    // client.open();

    // std::thread::sleep(Duration::from_secs(1));

    std::thread::spawn(move || {
        client.open();
        // std::thread::sleep(Duration::from_millis(500));
        // client.read_block();
        println!("Starting client polling");
        loop {
            println!("Polling...");
            client.poll().unwrap();

            match client.state() {
                // IoClientState::OpenedIdle => {
                //     client.read_block();
                // }
                // IoClientState::OpenedIdle => {
                //     println!("Stopping");
                //     return;
                // }
                state => {
                    println!("Client state {:?}", state);
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    t.join().unwrap();
}
