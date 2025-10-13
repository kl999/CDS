use std::{
    collections::HashMap, sync::{Arc, Mutex}
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
        let mut collection = self.collection.lock()
            .map_err(|x| format!("col lock!\n{}", x))?;
        
        let cell = collection.get_mut(&key);

        if let Some(cell) = cell {
            cell.client_id = self.client_id;
            cell.version = cell.version + 1;
            cell.value = val;
        }
        else {
            collection.insert(key, Cell::new(self.client_id, val));
        }

        Ok(())
    }

    pub fn get_key(&self, key: String) -> Result<Option<String>, String> {
        let collection = self.collection.lock()
            .map_err(|x| format!("col lock!\n{}", x))?;
        let val = collection.get(&key);

        if let Some(val) = val {
            return Ok(Some(val.value.to_string()));
        }

        Ok(None)
    }
}

pub struct Cell {
    pub client_id: u32,
    pub version: u64,
    pub value: String
}

impl Cell {
    pub fn new(client_id: u32, value: String) -> Cell {
        Cell { client_id, version: 0, value }
    }
}
