use serde::{Deserialize, Serialize};

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Event {
    None = 0,
    Completed = 1,
    Started = 2,
    Stopped = 3,
}
