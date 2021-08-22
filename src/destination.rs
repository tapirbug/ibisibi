use crate::args::Destination;
use crate::telegram::Telegram;
use serialport::{new, DataBits, Parity, SerialPort, SerialPortBuilder, StopBits};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DestinationError>;

pub fn destination(destination: Destination) -> Result<()> {
    let mut serial = new(&destination.serial, 1200)
        .data_bits(DataBits::Seven)
        .stop_bits(StopBits::Two)
        .parity(Parity::Even)
        .open()
        .map_err(|e| DestinationError::serial(e, &destination.serial))?;

    let telegram = Telegram::destination(destination.index);

    serial
        .write(telegram.as_bytes())
        .map_err(|e| DestinationError::io(e, &destination.serial))?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum DestinationError {
    #[error("Could not send command to switch destination by index to port: {port}, due to I/O error: {source}")]
    IO {
        source: std::io::Error,
        port: String,
    },
    #[error("Could not open serial port connection to: {port}, due to error: {source}")]
    Serial {
        source: serialport::Error,
        port: String,
    },
}

impl DestinationError {
    fn io(source: std::io::Error, port: &str) -> Self {
        Self::IO {
            source,
            port: port.into(),
        }
    }

    fn serial(source: serialport::Error, port: &str) -> Self {
        Self::Serial {
            source,
            port: port.into(),
        }
    }
}
