use serde::{Deserialize, Serialize};

use crate::udp_server::action::Action;

const DEFAULT_ANNOUNCE_INTERVAL: u32 = 10 * 60 * 1000; // 10 minutes

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AnnounceResponse {
    action: Action,
    transaction_id: u32,
    interval: u32,
    leechers: u32,
    seeders: u32,
    ip_address: u32,
    port: u16,
}
