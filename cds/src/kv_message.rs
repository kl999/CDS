use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KVMessage {
    pub key: String,
    pub client_id: u32,
    pub version: u64,
    pub value: String,
}

impl KVMessage {
    pub fn new(key: String, value: String, client_id: u32, version: u64) -> KVMessage {
        KVMessage {
            key,
            client_id,
            version,
            value,
        }
    }
}
