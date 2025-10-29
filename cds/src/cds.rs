use std::{
    collections::{HashMap, VecDeque}, sync::{Arc, Mutex}, thread
};

use crate::peer::Peer;

pub struct Cds {
    pub client_id: u32,
    peer_map: Vec<PeerMapItem>,
    collection: Arc<Mutex<HashMap<String, Cell>>>,
    keys_to_push: Arc<Mutex<VecDeque<(String, String, u64)>>>,
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl Cds {
    pub fn new(client_id: u32) -> Result<Arc<Cds>, String> {
        let cds = Cds {
            client_id,
            peer_map: vec!(),
            collection: Arc::new(Mutex::new(HashMap::new())),
            keys_to_push: Arc::new(Mutex::new(VecDeque::with_capacity(50))),
            peers: Arc::new(Mutex::new(vec![])),
        };

        let cds = Arc::new(cds);
        /*let in_thread = Arc::clone(&cds);

        thread::spawn(move || {
            in_thread.work();
        });*/

        Ok(cds)
    }

    pub fn set_key(&self, key: String, val: String) -> Result<(), String> {
        let mut collection = self
            .collection
            .lock()
            .map_err(|x| format!("col lock!\n{}", x))?;

        let cell = collection.get_mut(&key);
        let mut ver = 0;

        if let Some(cell) = cell {
            cell.client_id = self.client_id;
            cell.version = cell.version + 1;
            cell.value = val.clone();
            ver = cell.version;
        } else {
            collection.insert(key.clone(), Cell::new(self.client_id, val.clone()));
        }

        let mut keys_to_push = self.keys_to_push
            .lock()
            .map_err(|x| format!("keys push lock!\n{}", x))?;
        
        keys_to_push.push_back((key, val, ver));

        Ok(())
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
            collection.insert(key, Cell{ client_id, version, value: val });
        }

        Ok(())
    }

    pub fn get_key(&self, key: String) -> Result<Option<String>, String> {
        let collection = self
            .collection
            .lock()
            .map_err(|x| format!("col lock!\n{}", x))?;
        let val = collection.get(&key);

        if let Some(val) = val {
            return Ok(Some(val.value.to_string()));
        }

        Ok(None)
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

    fn work(&self) -> Result<(), String> {
        self.regenerate_peers();

        //TODO: handle errors!
        self.push_keys_to_peers()?;

        let peers = self.peers.clone();
        let mut peers = peers
            .lock()
            .map_err(|x| format!("peers lock!\n{}", x))?;

        for peer in &mut *peers {
            peer.work(self)?;
        }

        Ok(())
    }

    fn push_keys_to_peers(&self)-> Result<(), String> {
        let mut keys_to_push = self.keys_to_push
            .lock()
            .map_err(|x| format!("keys push lock!\n{}", x))?;
        while let Some((key, value, version)) = keys_to_push.pop_front() {
            let mut peers = self.peers
                .lock()
                .map_err(|x| format!("peers lock!\n{}", x))?;
            for peer in &mut *peers {
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
