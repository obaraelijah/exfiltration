use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum Messages {
    Initial {
        file_name: String,
        file_size: u64,
        identifier: u16,
    },
    Data {
        identifier: u16,
        data: Vec<u8>,
    },
    EndOfTransmission {
        identifier: u16,
    },
}
