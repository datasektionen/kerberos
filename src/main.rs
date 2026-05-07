use log::info;
use std::thread;

mod cli;
mod onboard;
mod reader;

enum Event {
    Card(String),
}

fn main() {
    env_logger::init();

    let mut state = cli::State::init();

    let (tx, rx) = std::sync::mpsc::channel::<Event>();

    info!("Starting reader thread");
    thread::spawn(move || {
        let mut reader = reader::Reader::new().expect("failed to create reader");

        loop {
            reader.wait_for_change();
            let uids = reader.status_loop();
            for uid in uids {
                tx.send(Event::Card(uid))
                    .expect("Failed to send card event");
            }
        }
    });

    let client = reqwest::blocking::Client::new();
    info!(
        "Starting main loop, sending card events to server at {}",
        &state.server_url
    );
    println!("Waiting for cards...");
    for event in rx {
        match event {
            Event::Card(uid) => {
                info!("Received card event with UID: {}", uid);
                println!("Received card event with UID: {}", uid);
                onboard::send_card_or_onboard(
                    &client,
                    &state.server_url,
                    &state.key,
                    uid,
                    state.onboard_only,
                    state.file.as_mut(),
                )
                .expect("Failed to send card event or onboard card");
            }
        }
    }
}
