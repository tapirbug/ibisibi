use crate::args::{Cycle, Destination};
use crate::destination::{destination, DestinationError};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

type Result<T> = std::result::Result<T, CycleError>;

const RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub fn cycle(options: &Cycle) -> Result<()> {
    assert!(options.interval_secs > 1.0, "Expected at least 1s delay");
    assert!(
        !options.indexes.is_empty(),
        "Expected at least one destination index"
    );

    let indexes = options.indexes.iter().flat_map(|r| r.iter()).cycle();
    let sleep_duration = Duration::from_secs_f64(options.interval_secs);

    for destination_index in indexes {
        let destination_args = Destination {
            index: destination_index as u16,
            serial: options.serial.clone(),
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
