use std::{
    collections::HashMap, sync::{Arc, Mutex}
};

pub struct Cds {
    collection: Arc<Mutex<HashMap<String, String>>>,
}

impl Cds {
    pub fn new() -> Result<Cds, String> {
        Ok(Cds {
            collection: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_key(&self, key: String) -> Result<Option<String>, String> {
        let collection = self.collection.lock()
            .map_err(|x| format!("col lock!\n{}", x))?;
        let val = collection.get(&key);

        if let Some(val) = val {
            return Ok(Some(val.to_string()));
        }

        Ok(None)
    }

    pub fn set_key(&mut self, key: String, val: String) -> Result<(), String> {
        let mut collection = self.collection.lock()
            .map_err(|x| format!("col lock!\n{}", x))?;
        
        collection.insert(key, val);

        Ok(())
    }
}
