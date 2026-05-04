mod reader;

fn main() {
    env_logger::init();

    let mut reader = reader::Reader::new().expect("failed to create reader");

    loop {
        reader.wait_for_change().expect("failed to wait for change");
        let uids = reader.status_loop();
        for uid in uids {
            println!("{uid}");
        }
    }
}
