use crate::args::Cycle;
use crate::telegram::Telegram;
use serialport::{new, DataBits, Parity, StopBits};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

type Result<T> = std::result::Result<T, CycleError>;

pub fn cycle(options: Cycle) -> Result<()> {
    assert!(options.interval_secs > 1.0, "Expected at least 1s delay");
    assert!(
        options.indexes.len() >= 1,
        "Expected at least one destination index"
    );
    assert!(
        options.from <= options.to,
        "First destination needs to be before last destination to show"
    );
    assert!(options.from < std::u16::MAX.into());
    assert!(options.to < std::u16::MAX.into());
    for idx in &options.indexes {
        assert!(*idx < std::u16::MAX.into());
    }

    let mut serial = new(&options.serial, 1200)
        .data_bits(DataBits::Seven)
        .stop_bits(StopBits::Two)
        .parity(Parity::Even)
        .open()
        .map_err(|e| CycleError::serial(e, &options.serial))?;

    let indexes = options
        .indexes
        .iter()
        .cloned()
        .chain((options.from)..(options.to))
        .cycle();

    let sleep_duration = Duration::from_secs_f64(options.interval_secs);

    for destination_index in indexes {
        let telegram = Telegram::destination(destination_index as u16);
        serial
            .write(telegram.as_bytes())
            .map_err(|e| CycleError::io(e, &options.serial))?;

        sleep(sleep_duration);
    }

    // TODO ctrl+c and successful return
    unreachable!("loop never terminates because of the cycle on the iterator")
}

#[derive(Error, Debug)]
pub enum CycleError {
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

impl CycleError {
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
