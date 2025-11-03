use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc::{self, Sender}},
    thread::{self, JoinHandle},
};

use crate::cds_worker::{CdsWorker, Cell};

pub struct Cds {
    collection: Arc<Mutex<HashMap<String, Cell>>>,
    worker_handle: JoinHandle<()>,
    tx: Sender<(String, String)>,
}

impl Cds {
    pub fn new(client_id: u32) -> Result<Cds, String> {
        let collection = Arc::new(Mutex::new(HashMap::new()));

        let collection_thread = Arc::clone(&collection);

        let (tx, rx) = mpsc::channel();

        let worker_handle = thread::spawn(move || {
            let cds_worker = CdsWorker::new(client_id, collection_thread, rx);
            cds_worker.work();
        });

        Ok(Cds {
            collection,
            worker_handle,
            tx,
        })
    }

    pub fn set_key(&self, key: String, val: String) -> Result<(), String> {
        self.tx.send((key, val))
        .map_err(|x| format!("rx send {}", x))?;

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
