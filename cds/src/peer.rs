use udp_connection::SocketWorker;

use crate::kv_message::KVMessage;

pub struct Peer {
    connect: SocketWorker,
    is_dead: bool,
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

    pub(crate) fn work(&mut self, cds: &mut crate::cds::Cds) -> Result<(), String> {
        let msgs = self.connect.work();

        for msg in msgs {
            process_message(cds, msg)?;
        }

        Ok(())
    }
}

fn process_message(control: &mut crate::cds::Cds, msg: Box<[u8]>) -> Result<(), String> {
    let msg = String::from_utf8(msg.to_vec())
        .map_err(|x| format!("To String!\n{}", x))?;
    let msg: KVMessage = serde_json::from_str(&msg)
        .map_err(|x| format!("From JSON!\n{}", x))?;

    //TODO: Check for client id? Mb it is forward?

    control.set_key_foreign(msg.key, msg.value, msg.client_id, msg.version)?;

    Ok(())
}
