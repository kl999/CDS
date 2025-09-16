use crate::cds::Cds;
use std::error::Error;

mod cds;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cds = Cds::new()?;

    let val = cds.get_key("A".to_string())?;

    println!("key 'A' = {}", val);

    cds.set_key("A".to_string(), "Hello".to_string())?;
    
    Ok(())
}
