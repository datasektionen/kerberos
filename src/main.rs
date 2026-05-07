use log::info;
use std::{io::Write, thread};

mod cli;
mod onboard;
mod reader;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
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
            reader.wait_for_change().expect("failed to wait for change");
            let uids = reader.status_loop();
            for uid in uids {
                if let Some(file) = &mut state.file {
                    info!("Wrote UID to file: {}", uid);
                    writeln!(file, "{}", uid).expect("Failed to write UID to file");
                } else {
                    info!("Received card event with UID: {}", uid);
                }
                tx.send(Event::Card(uid))
                    .expect("Failed to send card event");
            }
        }
    });

    let client = reqwest::blocking::Client::new();
    let server_url = format!("{}/card", &state.server_url);
    info!(
        "Starting main loop, sending card events to server at {}",
        server_url
    );
    for event in rx {
        match event {
            Event::Card(uid) => {
                info!("Received card event with UID: {}", uid);
                onboard::send_card_or_onboard(
                    &client,
                    &server_url,
                    &state.key,
                    uid,
                    state.onboard_only,
                )
                .expect("Failed to send card event or onboard card");
            }
        }
    }
}
