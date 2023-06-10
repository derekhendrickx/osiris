use serde::{Deserialize, Serialize};

use crate::udp_server::action::Action;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ErrorResponse {
    action: Action,
    transaction_id: u32,
    error_string: String,
}
