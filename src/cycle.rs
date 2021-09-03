use crate::args::{Cycle, Destination};
use crate::destination::{destination, DestinationError};
use crate::plan::Plan;
use crate::slot::Slot;
use chrono::{Duration as ChronoDuration, Local};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

type Result<T> = std::result::Result<T, CycleError>;

const RETRY_INTERVAL: Duration = Duration::from_secs(5);

pub fn cycle(options: &Cycle) -> Result<()> {
    assert!(options.interval_secs > 1.0, "Expected at least 1s delay");
    assert!(
        !options.plan.is_empty(),
        "Expected at least one destination index"
    );

    let sleep_duration = Duration::from_secs_f64(options.interval_secs);
    let lookahead = ChronoDuration::hours(options.lookahead as i64);
    loop {
        let active_count = options
            .plan
            .iter()
            .filter(|plan| is_active(plan.slots(), lookahead))
            .map(|plan| execute(plan, &options.serial, sleep_duration))
            .count();
        if active_count == 0 {
            eprintln!(
                "nothing to show at the moment, retry after {interval:?}",
                interval = RETRY_INTERVAL
            );
            sleep(RETRY_INTERVAL);
        }
    }
}

/// Checks whether the given plan element applies at the current point
/// in time, executes the plan, and returns whether or not it had applied.
///
/// When errors occur, e.g. serial port disconnection, then retries until
/// successful execution.
fn execute(plan: &Plan, serial: &str, sleep_duration: Duration) {
    let line = plan.line();
    let destinations = plan.destinations().iter().flat_map(|r| r.iter());

    for destination_index in destinations {
        let destination_args = Destination {
            index: destination_index as u16,
            line,
            serial: serial.to_string(),
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
}

fn is_active(slots: &[Slot], lookahead: ChronoDuration) -> bool {
    if slots.is_empty() {
        return true; // no slots defined means show always
    }

    let now = Local::now().naive_local();
    let soonest_to_show = now + lookahead;
    slots.iter().any(|slot| {
        // cease to show events when already over
        now < slot.end()
                // show when currently happening or within lookahead
                && soonest_to_show > slot.start()
    })
}

#[derive(Error, Debug)]
pub enum CycleError {
    #[error("{0}")]
    Destination(#[from] DestinationError),
}
