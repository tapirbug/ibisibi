use crate::args::{Cycle, Destination};
use crate::destination::{destination, DestinationError};
use crate::telegram::Telegram;
use serialport::{new, DataBits, Parity, StopBits};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

type Result<T> = std::result::Result<T, CycleError>;

const RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub fn cycle(options: &Cycle) -> Result<()> {
    assert!(options.interval_secs > 1.0, "Expected at least 1s delay");
    assert!(
        options.indexes.len() >= 1 || options.from < options.to,
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

    let indexes = options
        .indexes
        .iter()
        .cloned()
        .chain((options.from)..(options.to))
        .cycle();

    let sleep_duration = Duration::from_secs_f64(options.interval_secs);

    for destination_index in indexes {
        let destination_args = Destination {
            index: destination_index as u16,
            serial: options.serial.clone()
        };
        while let Err(err) = destination(&destination_args) {
            eprintln!(
                "error: could not switch to destination {dest}, reason: {reason}, retry after {interval:?}",
                dest = destination_index,
                reason = err,
                interval = RETRY_INTERVAL
            );
            sleep(RETRY_INTERVAL);
        }
        sleep(sleep_duration);
    }

    // TODO ctrl+c and successful return
    unreachable!("loop never terminates because of the cycle on the iterator")
}

#[derive(Error, Debug)]
pub enum CycleError {
    #[error("{0}")]
    Destination(#[from] DestinationError),
}
