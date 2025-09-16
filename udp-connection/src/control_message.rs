use std::fmt::Debug;

#[derive(Debug)]
pub enum ControlMessage {
    Acc { id: u64 },
}
