use crate::args::List;
use thiserror::Error;

pub fn list(_list: List) -> Result<(), ListError> {
    let ports = serialport::available_ports()?;
    for p in ports {
        println!("{}", p.port_name);
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum ListError {
    #[error("Could not list serial ports: {0}")]
    Serial(#[from] serialport::Error),
}
