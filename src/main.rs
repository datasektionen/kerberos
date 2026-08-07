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
                info!("Received card event with UID: {}", &uid);
                println!("Received card: {}", &uid);

                'main: for i in 0..3 {
                    let result = onboard::CardRequestBuilder::new(uid.clone(), &state.server_url)
                        .token(&state.key)
                        .file(state.file.as_mut())
                        .onboard(state.onboard_only)
                        .build(&client);
                    match result {
                        onboard::RequestResult::Success => {
                            log::info!("Card event sent successfully");
                            println!("Success");
                            break;
                        },
                        onboard::RequestResult::NotFound => {
                            log::info!("Card not found, onboarding...");
                            println!("Card with UID {} not found, onboarding...", uid);

                            for i in 0..3 {
                                let result = onboard::CardRequestBuilder::new(
                                    uid.clone(),
                                    &state.server_url,
                                )
                                .token(&state.key)
                                .file(state.file.as_mut())
                                .onboard(true)
                                .build(&client);
                                match result {
                                    onboard::RequestResult::Success => break 'main,
                                    onboard::RequestResult::NotFound => {
                                        log::error!("Card not found after onboarding, giving up");
                                        break 'main;
                                    }
                                    onboard::RequestResult::RateLimited => {
                                        log::warn!(
                                            "Rate limited by server during onboarding, retrying in 3 seconds... ({}/3)",
                                            i + 1
                                        );
                                        thread::sleep(std::time::Duration::from_secs(3));
                                        continue;
                                    }
                                    onboard::RequestResult::NotMember(kthid) => {
                                        log::info!("Card holder not a member, kthid: {}", kthid);
                                        println!("Card holder not a member, kthid: {}", kthid);
                                        break 'main;
                                    }
                                    onboard::RequestResult::MeetingNotActive => {
                                        log::info!("Meeting is not active");
                                        println!(
                                            "Meeting is not active, active the meeting on {}",
                                            &state.server_url
                                        );
                                        break 'main;
                                    }
                                    onboard::RequestResult::OnboardConflict(kthid) => {
                                        log::info!(
                                            "Card already onboarded to another user, kthid: {}",
                                            kthid
                                        );
                                        println!(
                                            "Card already onboarded to another user, kthid: {}",
                                            kthid
                                        );
                                        break 'main;
                                    }
                                    onboard::RequestResult::TokenNotFound => {
                                        log::error!(
                                            "Token not found, create a token on {}",
                                            &state.server_url
                                        );
                                        break 'main;
                                    }
                                    onboard::RequestResult::Error(e) => {
                                        log::error!(
                                            "Error sending card event to server during onboarding: {}",
                                            e
                                        );
                                        break 'main;
                                    }
                                }
                            }
                        }
                        onboard::RequestResult::RateLimited => {
                            log::warn!(
                                "Rate limited by server, retrying in 3 seconds... ({}/3)",
                                i + 1
                            );
                            thread::sleep(std::time::Duration::from_secs(3));
                            continue;
                        }
                        onboard::RequestResult::NotMember(kthid) => {
                            log::info!("Card holder not a member, kthid: {}", kthid);
                            println!("Card holder not a member, kthid: {}", kthid);
                            break;
                        }
                        onboard::RequestResult::MeetingNotActive => {
                            log::info!("Meeting is not active");
                            println!(
                                "Meeting is not active, active the meeting on {}",
                                &state.server_url
                            );
                            break;
                        }
                        onboard::RequestResult::TokenNotFound => {
                            log::error!("Token not found, create a token on {}", &state.server_url);
                            break;
                        }
                        onboard::RequestResult::OnboardConflict(e)
                        | onboard::RequestResult::Error(e) => {
                            log::error!("Error sending card event to server: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}
