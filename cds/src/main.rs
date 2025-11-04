use crate::cds::Cds;

mod cds;
mod peer;
mod kv_message;
mod cds_worker;

fn main() -> Result<(), String> {
    let cds = Cds::new(1)?;

    let val = cds.get_key("A".to_string())?;

    println!("key 'A' = {:?}", val);

    cds.set_key("A".to_string(), "Hello".to_string())?;

    let val = cds.get_key("A".to_string())?;

    println!("key 'A' = {:?}", val);
    
    cds.stop();

    Ok(())
}
