use udp_connection::{SocketWorker, send_handshake};

use crate::kv_message::KVMessage;

pub struct Peer {
    pub address: String,
    pub id: u32,
    pub is_dead: bool,
    connect: SocketWorker,
}

impl Peer {
    pub(crate) fn new(address: String, id: u32) -> Result<Peer, String> {
        let connect =
            send_handshake(address.clone(), |_| {}).map_err(|x| format!("send_handshake {}", x))?;

        Ok(Peer {
            address,
            id,
            is_dead: false,
            connect,
        })
    }

    pub(crate) fn new_from_worker(address: String, id: u32, worker: SocketWorker) -> Peer {
        Peer {
            address,
            id,
            is_dead: false,
            connect: worker,
        }
    }

    pub(crate) fn push_val(
        &mut self,
        key: String,
        value: String,
        client_id: u32,
        version: u64,
    ) -> Result<(), String> {
        let message = KVMessage::new(key, value, client_id, version);

        let message = serde_json::to_string(&message).map_err(|x| format!("To JSON! {}", x))?;

        self.connect
            .send_message(message.as_bytes().to_vec().into_boxed_slice());

        Ok(())
    }

    pub(crate) fn work(&mut self) -> Result<Vec<PeerResult>, String> {
        let msgs = self.connect.work();
        let mut results = vec![];

        for msg in msgs {
            match msg {
                Ok(msg) => results.push(process_message(msg)?),
                Err(e) => {
                    eprintln!("Error from peer {e}");
                    self.die();
                    break;
                }
            }
        }

        Ok(results)
    }

    fn die(&mut self) {
        self.is_dead = true;
    }
}

fn process_message(msg: Box<[u8]>) -> Result<PeerResult, String> {
    let msg = String::from_utf8(msg.to_vec()).map_err(|x| format!("To String! {}", x))?;
    let msg: KVMessage = serde_json::from_str(&msg).map_err(|x| format!("From JSON! {}", x))?;

    // TODO: Check for client id? Mb it is forward?

    Ok(PeerResult::KeyUpdate(
        msg.key,
        msg.value,
        msg.version,
        msg.client_id,
    ))
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum PeerResult {
    Unknown,
    /// KeyUpdate(key, value, version, client_id)
    KeyUpdate(String, String, u64, u32),
}
