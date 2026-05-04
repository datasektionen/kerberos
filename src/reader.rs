use log::{info, warn};
use pcsc::*;

pub struct Reader {
    ctx: Context,
    states: Vec<ReaderState>,
    buffer: [u8; 256],
}

impl Reader {
    pub fn new() -> Result<Self, String> {
        let ctx = Context::establish(Scope::User)
            .map_err(|e| format!("failed to establish context: {}", e))?;

        let reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
        ];

        Ok(Self {
            ctx,
            states: reader_states,
            buffer: [0; 256],
        })
    }

    pub fn wait_for_change(&mut self) -> Result<(), String> {
        self.refresh_readers();

        self.ctx
            .get_status_change(None, &mut self.states)
            .map_err(|e| format!("failed to get status change: {}", e))
    }

    fn refresh_readers(&mut self) {
        // Remove dead readers.
        fn is_dead(rs: &ReaderState) -> bool {
            rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
        }

        for rs in &self.states {
            if is_dead(rs) {
                info!("Removing reader: {:?}", rs.name());
            }
        }
        self.states.retain(|rs| !is_dead(rs));

        // Add new readers.
        let names = self
            .ctx
            .list_readers(&mut self.buffer)
            .expect("failed to list readers");
        for name in names {
            if !self.states.iter().any(|rs| rs.name() == name) {
                info!("Adding reader: {:?}", name);
                self.states.push(ReaderState::new(name, State::UNAWARE));
            }
        }

        // Update the view of the state to wait on.
        for rs in &mut self.states {
            rs.sync_current_state();
        }
    }

    pub fn status_loop(&self) -> Vec<String> {
        let mut uids = Vec::new();
        for rs in &self.states {
            if rs.name() == PNP_NOTIFICATION() {
                continue;
            }
            info!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());
            
            // If the reader reports a card present, connect and request UID.
            if rs.event_state().contains(State::PRESENT) {
                match self.ctx.connect(rs.name(), ShareMode::Shared, Protocols::ANY) {
                    Ok(card) => {
                        info!("Connected to {:?}, sending APDU command", rs.name());
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
                                    info!("\tUID found: {}", s);
                                    uids.push(s);
                                } else {
                                    // Some cards/readers return other SWs or data; still show raw response.
                                    log::error!("\tResponse (len={}): {:02X?}", l, &recv[..l]);
                                }
                            }
                            Ok(_) => warn!("Transmit returned no data"),
                            Err(e) => {
                                log::error!("Failed to transmit APDU to {:?}: {:?}", rs.name(), e)
                            }
                        }
                    }
                    Err(e) => log::error!("Failed to connect to {:?}: {:?}", rs.name(), e),
                }
            }
        }

        uids
    }
}
