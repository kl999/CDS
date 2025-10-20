use crate::cds::Cds;

mod cds;
mod peer;
mod kv_message;

fn main() -> Result<(), String> {
    let mut cds = Cds::new(1)?;

    let val = cds.get_key("A".to_string())?;

    println!("key 'A' = {:?}", val);

    cds.set_key("A".to_string(), "Hello".to_string())?;

    let val = cds.get_key("A".to_string())?;

    println!("key 'A' = {:?}", val);
    
    Ok(())
}
