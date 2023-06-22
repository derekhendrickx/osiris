use serde::{Deserialize, Serialize};

use super::headers::Headers;
use crate::udp_server::{event::Event, payload::Payload};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AnnounceRequest {
    header: Headers,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    downloaded: u64,
    left: u64,
    uploaded: u64,
    event: Event,
    ip_address: u32,
    key: u32,
    num_want: u32,
    port: u16,
}

impl Payload for AnnounceRequest {
    fn hello(&self) {
        todo!()
    }
}
