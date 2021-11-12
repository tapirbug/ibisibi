use thiserror::Error;
use crate::{
    args::Scan as Opts,
    scan::Scan
};
use serialport::{new, DataBits, Parity, StopBits};

type Result<T> = std::result::Result<T, ScanError>;

pub fn scan(scan: Opts) -> Result<()> {
    let mut serial = new(&scan.serial, 1200)
        .data_bits(DataBits::Seven)
        .stop_bits(StopBits::Two)
        .parity(Parity::Even)
        .open()
        .map_err(|e| ScanError::Serial { source: e, port: scan.serial })?;

    let mut none = false;
    for find in Scan::new(&mut serial).filter_map(crate::scan::Result::ok) {
        none = true;
        println!(
            "{address:X?}: {status}",
            address = find.address(),
            status = find.status()
        );
    }
    if none {
        println!("No display devices found.")
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Could not open serial port connection to: {port}, due to error: {source}")]
    Serial {
        source: serialport::Error,
        port: String,
    },
}