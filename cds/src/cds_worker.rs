use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use crate::peer::{Peer, PeerResult};

pub struct CdsWorker {
    pub client_id: u32,
    peer_map: Vec<PeerMapItem>,
    collection: Arc<Mutex<HashMap<String, Cell>>>,
    keys_to_push: VecDeque<(String, String, u64)>,
    peers: Vec<Peer>,
}

impl CdsWorker {
    pub fn new(client_id: u32, collection: Arc<Mutex<HashMap<String, Cell>>>) -> CdsWorker {
        CdsWorker {
            client_id,
            peer_map: vec![],
            collection,
            keys_to_push: VecDeque::with_capacity(50),
            peers: vec![],
        }
    }

    pub fn set_key_foreign(
        &self,
        key: String,
        val: String,
        client_id: u32,
        version: u64,
    ) -> Result<(), String> {
        let mut collection = self
            .collection
            .lock()
            .map_err(|x| format!("col lock!\n{}", x))?;

        let cell = collection.get_mut(&key);

        if let Some(cell) = cell {
            if version > cell.version || version == cell.version && client_id < self.client_id {
                cell.client_id = client_id;
                cell.version = version;
                cell.value = val;
            }
            //else version < cell.version do nothing
        } else {
            collection.insert(
                key,
                Cell {
                    client_id,
                    version,
                    value: val,
                },
            );
        }

        Ok(())
    }

    //fn define_client_id(&mut self){}
    // Workflow:
    // 1. Connect to ANY bootstrap peer
    // 2. Request peer list
    // 3. Find peer with highest ID
    // 4. Connect to that peer
    // 5. Request ID assignment
    // 6. Get new_id = max_id + 1
    // 7. Announce yourself to all peers
    // 8. You are now the "last" peer (until someone else joins)

    pub fn work(mut self) {
        loop {
            self.regenerate_peers();

            //TODO: handle errors!
            if let Err(e) = self.push_keys_to_peers() {
                eprintln!("Push error: {}", e);
            }

            for peer in &mut self.peers {
                match peer.work() {
                    Ok(result) => self.consume_peer_result(result),
                    Err(e) => eprintln!("Peer work error: {}", e),
                }
            }
        }
    }

    fn consume_peer_result(&mut self, result: Vec<PeerResult>) {

    }

    fn push_keys_to_peers(&mut self) -> Result<(), String> {
        while let Some((key, value, version)) = self.keys_to_push.pop_front() {
            for peer in &mut self.peers {
                peer.push_val(key.clone(), value.clone(), self.client_id, version)?;
            }
        }

        Ok(())
    }

    fn regenerate_peers(&self) {
        self.accept_new_peer();
        self.regenerate_from_map();
    }

    fn accept_new_peer(&self) {}
    fn regenerate_from_map(&self) {
        for item in &self.peer_map {
            //if peer not found make new from map
        }
    }
}

pub struct Cell {
    pub client_id: u32,
    pub version: u64,
    pub value: String,
}

impl Cell {
    pub fn new(client_id: u32, value: String) -> Cell {
        Cell {
            client_id,
            version: 0,
            value,
        }
    }
}

struct PeerMapItem {
    pub address: String,
    pub client_id: u32,
}
