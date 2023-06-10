use serde::{Deserialize, Serialize};

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Action {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
    Error = 3,
}
