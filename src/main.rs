use log::info;
use std::io::Write;

mod cli;
mod reader;

fn main() {
    env_logger::init();

    let mut state = cli::State::init();

    let mut reader = reader::Reader::new().expect("failed to create reader");

    loop {
        reader.wait_for_change().expect("failed to wait for change");
        let uids = reader.status_loop();
        for uid in uids {
            if let Some(file) = &mut state.file {
                info!("Wrote UID to file: {}", uid);
                writeln!(file, "{}", uid).expect("Failed to write UID to file");
                println!("Found UID: {}", uid);
            } else {
                println!("{}", uid);
            }
        }
    }
}
