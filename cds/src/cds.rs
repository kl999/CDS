use std::error::Error;

pub struct Cds {

}

impl Cds {
    pub fn new() -> Result<Cds, Box<dyn Error>> {
        Ok(Cds {})
    }
    
    pub fn get_key(&self, key: String) -> Result<String, Box<dyn Error>> {
        // Example of creating a dynamic error
        Err(format!("Not implemented").into())
    }
    
    pub fn set_key(&self, key: String, val: String) -> Result<(), Box<dyn Error>> {
        // Another example
        Err("Storage not implemented yet".into())
    }
}