use serde::{Deserialize, Serialize};

use crate::udp_server::action::Action;

// Magic constant from https://www.bittorrent.org/beps/bep_0015.html
pub const PROTOCOL_ID: u64 = 0x41727101980;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Headers {
    connection_id: u64,
    action: Action,
    transaction_id: u32,
}

impl Headers {
    pub fn new(connection_id: u64, action: Action, transaction_id: u32) -> Headers {
        Headers {
            connection_id: connection_id,
            action: action,
            transaction_id: transaction_id
        }
    }

    pub fn is_torrent(&self) -> bool {
        self.connection_id == PROTOCOL_ID
    }

    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn transaction_id(&self) -> u32 {
        self.transaction_id
    }
}

#[cfg(test)]
mod tests {
    use fake::{Faker, Fake};

    use super::{Action, Headers, PROTOCOL_ID};

    #[test]
    fn test_it_creates_headers() {
        let connection_id = PROTOCOL_ID;
        let action = Action::Connect;
        let transaction_id = Faker.fake::<u32>();

        let headers = Headers::new(
            connection_id,
            action,
            transaction_id
        );

        assert_eq!(headers.connection_id, connection_id);
        assert_eq!(headers.action, Action::Connect);
        assert_eq!(headers.transaction_id, transaction_id);
    }

    #[test]
    fn test_it_check_is_torrent_returns_true() {
        let headers = Headers::new(
            PROTOCOL_ID,
            Action::Connect,
            Faker.fake::<u32>()
        );

        let result = headers.is_torrent();

        assert_eq!(result, true);
    }

    #[test]
    fn test_it_check_is_torrent_returns_false() {
        let headers = Headers::new(
            Faker.fake::<u64>(),
            Action::Connect,
            Faker.fake::<u32>()
        );

        let result = headers.is_torrent();

        assert_eq!(result, false);
    }
}
