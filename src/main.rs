use pcsc::*;

fn main() {
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    let mut readers_buf = [0; 2048];
    let mut reader_states = vec![
        // Listen for reader insertions/removals, if supported.
        ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
    ];
    loop {
        // Remove dead readers.
        fn is_dead(rs: &ReaderState) -> bool {
            rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
        }
        for rs in &reader_states {
            if is_dead(rs) {
                println!("Removing {:?}", rs.name());
            }
        }
        reader_states.retain(|rs| !is_dead(rs));

        // Add new readers.
        let names = ctx
            .list_readers(&mut readers_buf)
            .expect("failed to list readers");
        for name in names {
            if !reader_states.iter().any(|rs| rs.name() == name) {
                println!("Adding {:?}", name);
                reader_states.push(ReaderState::new(name, State::UNAWARE));
            }
        }

        // Update the view of the state to wait on.
        for rs in &mut reader_states {
            rs.sync_current_state();
        }

        // Wait until the state changes.
        ctx.get_status_change(None, &mut reader_states)
            .expect("failed to get status change");

        // Print current state and, if a card is present, try to read its UID.
        println!();
        for rs in &reader_states {
            if rs.name() == PNP_NOTIFICATION() {
                continue;
            }
            println!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());

            // If the reader reports a card present, connect and request UID.
            if rs.event_state().contains(State::PRESENT) {
                match ctx.connect(rs.name(), ShareMode::Shared, Protocols::ANY) {
                    Ok(card) => {
                        // APDU commonly used to get the UID via PC/SC
                        let get_uid_apdu = [0xFF, 0xCA, 0x00, 0x00, 0x00];
                        let mut recv = [0u8; 256];
                        match card.transmit(&get_uid_apdu, &mut recv) {
                            Ok(len) if len.len() >= 2 => {
                                let l = len.len();
                                // Last two bytes are SW1 SW2 (e.g. 0x90 0x00).
                                let sw1 = recv[l - 2];
                                let sw2 = recv[l - 1];
                                if sw1 == 0x90 && sw2 == 0x00 {
                                    let uid_bytes = &recv[..l - 2];
                                    let s = uid_bytes
                                        .iter()
                                        .map(|b| format!("{:02X}", b))
                                        .collect::<Vec<String>>()
                                        .join(":");
                                    println!("UID: {}", s);
                                } else {
                                    // Some cards/readers return other SWs or data; still show raw response.
                                    println!("Response (len={}): {:02X?}", l, &recv[..l]);
                                }
                            }
                            Ok(_) => println!("Transmit returned no data"),
                            Err(e) => {
                                println!("Failed to transmit APDU to {:?}: {:?}", rs.name(), e)
                            }
                        }
                    }
                    Err(e) => println!("Failed to connect to {:?}: {:?}", rs.name(), e),
                }
            }
        }
    }
}
