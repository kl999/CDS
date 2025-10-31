use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::cds_worker::{CdsWorker, Cell};

pub struct Cds {
    collection: Arc<Mutex<HashMap<String, Cell>>>,
    worker_handle: JoinHandle<()>,
}

impl Cds {
    pub fn new(client_id: u32) -> Result<Cds, String> {
        let collection = Arc::new(Mutex::new(HashMap::new()));

        let collection_thread = Arc::clone(&collection);

        let worker_handle = thread::spawn(move || {
            let cds_worker = CdsWorker::new(client_id, collection_thread);
            cds_worker.work();
        });

        Ok(Cds {
            collection,
            worker_handle,
        })
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

        let mut keys_to_push = self
            .keys_to_push
            .lock()
            .map_err(|x| format!("keys push lock!\n{}", x))?;

        keys_to_push.push_back((key, val, ver));

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

    pub fn stop(self) {
        // do stuff like worker.stop()
        self.worker_handle.join();
    }
}
