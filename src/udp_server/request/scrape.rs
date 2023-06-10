use serde::{Deserialize, Serialize};

use super::headers::Headers;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ScrapeRequest {
    header: Headers,
    info_hash: [u8; 20],
}
