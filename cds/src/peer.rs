use udp_connection::SocketWorker;

use crate::kv_message::KVMessage;

pub struct Peer {
    connect: SocketWorker,
}

impl Peer {
    pub(crate) fn push_val(
        &mut self,
        key: String,
        value: String,
        client_id: u32,
        version: u64,
    ) -> Result<(), String> {
        let message = KVMessage::new(key, value, client_id, version);

        let message = serde_json::to_string(&message).map_err(|x| format!("To JSON!\n{}", x))?;

        self.connect
            .send_message(message.as_bytes().to_vec().into_boxed_slice());
        
        Ok(())
    }

    pub(crate) fn work(&mut self) -> Result<Vec<PeerResult>, String> {
        let msgs = self.connect.work();
        let mut results = vec![];

        for msg in msgs {
            results.push(process_message(msg)?);
        }

        Ok(results)
    }
}

fn process_message(msg: Box<[u8]>) -> Result<PeerResult, String> {
    let msg = String::from_utf8(msg.to_vec())
        .map_err(|x| format!("To String!\n{}", x))?;
    let msg: KVMessage = serde_json::from_str(&msg)
        .map_err(|x| format!("From JSON!\n{}", x))?;

    // TODO: Check for client id? Mb it is forward?
    
    Ok(PeerResult::KeyUpdate(msg.key, msg.value, msg.version, msg.client_id))
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum PeerResult {
    Unknown,
    /// key, value, version, client_id
    KeyUpdate(String, String, u64, u32),
}
