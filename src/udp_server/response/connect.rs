use serde::{Deserialize, Serialize};

use crate::udp_server::action::Action;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ConnectResponse {
    action: Action,
    transaction_id: u32,
    connection_id: u64,
}
