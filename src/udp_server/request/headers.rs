use serde::{Deserialize, Serialize};

use crate::udp_server::action::Action;

// Magic constant from https://www.bittorrent.org/beps/bep_0015.html
const PROTOCOL_ID: u64 = 0x41727101980;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Headers {
    connection_id: u64,
    action: Action,
    transaction_id: u32,
}

impl Headers {
    fn new(connection_id: u64, action: Action, transaction_id: u32) -> Headers {
        Headers {
            connection_id: connection_id,
            action: action,
            transaction_id: transaction_id
        }
    }

    fn is_torrent(&self) -> bool {
        self.connection_id == PROTOCOL_ID
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_is_torrent() {
        let headers = Headers {
            connection_id: PROTOCOL_ID,
            action: Action::Connect,
            transaction_id: 42
        };

        let result = headers.is_torrent();

        assert_eq!(result, true);
    }
}
