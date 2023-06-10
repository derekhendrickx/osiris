use serde::{Deserialize, Serialize};

use crate::udp_server::action::Action;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ScrapeResponse {
    action: Action,
    transaction_id: u32,
    info_hash: [u8; 20],
    seeders: u32,
    completed: u32,
    leechers: u32,
}
