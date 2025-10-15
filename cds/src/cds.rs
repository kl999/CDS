use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Cds {
    pub client_id: u32,
    collection: Arc<Mutex<HashMap<String, Cell>>>,
}

impl Cds {
    pub fn new(client_id: u32) -> Result<Cds, String> {
        Ok(Cds {
            client_id,
            collection: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn set_key(&mut self, key: String, val: String) -> Result<(), String> {
        let mut collection = self
            .collection
            .lock()
            .map_err(|x| format!("col lock!\n{}", x))?;

        let cell = collection.get_mut(&key);

        if let Some(cell) = cell {
            cell.client_id = self.client_id;
            cell.version = cell.version + 1;
            cell.value = val;
        } else {
            collection.insert(key, Cell::new(self.client_id, val));
        }

        Ok(())
    }

    fn set_key_foreign(
        &mut self,
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

    fn work(&mut self) {
        //see if key was added/updated

        //work on every connection
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
