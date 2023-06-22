use serde::{Deserialize, Serialize};

use crate::udp_server::payload::Payload;

use super::headers::Headers;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ScrapeRequest {
    header: Headers,
    info_hash: [u8; 20],
}

impl Payload for ScrapeRequest {
    fn hello(&self) {
        todo!()
    }
}
