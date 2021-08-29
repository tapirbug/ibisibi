//! A time slot.
use chrono::NaiveDateTime;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Slot {
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Slot {
    pub fn start(&self) -> NaiveDateTime {
        self.start
    }

    pub fn end(&self) -> NaiveDateTime {
        self.end
    }
}

impl FromStr for Slot {
    type Err = ParseSlotError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.is_empty() {
            return Err(ParseSlotError::Blank);
        }

        let mut dates = source.split('/');
        let start = dates
            .next()
            .ok_or_else(|| ParseSlotError::incomplete(source))?;
        let end = dates
            .next()
            .ok_or_else(|| ParseSlotError::incomplete(source))?;
        if dates.next().is_some() {
            return Err(ParseSlotError::too_much(source));
        }

        let start = start.parse::<NaiveDateTime>()?;
        let end = end.parse::<NaiveDateTime>()?;

        if start > end {
            return Err(ParseSlotError::from_after_to(start, end));
        }

        let slot = Slot { start, end };
        Ok(slot)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseSlotError {
    #[error("Could not parse blank string as a time slot")]
    Blank,
    #[error("Could not parse time slot from incomplete input `{input}`")]
    Incomplete { input: String },
    #[error("Time slot contains more than two dates: `{input}`")]
    TooMuch { input: String },
    #[error("Time slot end {end} was before time slot start {start}")]
    FromAfterTo {
        start: NaiveDateTime,
        end: NaiveDateTime,
    },
    #[error("Could not parse time part in timeslot: {0}")]
    DateFormat(#[from] chrono::ParseError),
}

impl ParseSlotError {
    fn incomplete(source: &str) -> Self {
        Self::Incomplete {
            input: source.to_string(),
        }
    }

    fn too_much(source: &str) -> Self {
        Self::TooMuch {
            input: source.to_string(),
        }
    }

    fn from_after_to(start: NaiveDateTime, end: NaiveDateTime) -> Self {
        Self::FromAfterTo { start, end }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn date_only() {
        let slot = "2021-09-09T00:00:00/2021-09-10T00:00:00"
            .parse::<Slot>()
            .unwrap();
        let expected_start = "2021-09-09T00:00:00".parse::<NaiveDateTime>().unwrap();
        let expected_end = "2021-09-10T00:00:00".parse::<NaiveDateTime>().unwrap();
        assert_eq!(
            slot,
            Slot {
                start: expected_start,
                end: expected_end
            }
        )
    }

    #[test]
    fn date_and_time() {
        let slot = "2021-09-09T20:00:00/2021-09-10T21:00:00"
            .parse::<Slot>()
            .unwrap();
        let expected_start = "2021-09-09T20:00:00".parse::<NaiveDateTime>().unwrap();
        let expected_end = "2021-09-10T21:00:00".parse::<NaiveDateTime>().unwrap();
        assert_eq!(
            slot,
            Slot {
                start: expected_start,
                end: expected_end
            }
        )
    }

    #[test]
    fn from_after_to() {
        let slot = "2021-09-19T20:00:00/2021-09-09T21:00:00"
            .parse::<Slot>()
            .unwrap_err();
        assert_eq!(
            slot,
            ParseSlotError::from_after_to(
                "2021-09-19T20:00:00".parse().unwrap(),
                "2021-09-09T21:00:00".parse().unwrap()
            )
        )
    }

    #[test]
    fn malformed_end_date() {
        match "2021-09-19T20:00:00/2021-09-0921:00:00"
            .parse::<Slot>()
            .unwrap_err()
        {
            ParseSlotError::DateFormat(_) => (),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn malformed_start_date() {
        match "2021-09-19T2000:00/2021-09-09T21:00:00"
            .parse::<Slot>()
            .unwrap_err()
        {
            ParseSlotError::DateFormat(_) => (),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn missing_slash() {
        match "2021-09-19T20:00:002021-09-09T21:00:00"
            .parse::<Slot>()
            .unwrap_err()
        {
            ParseSlotError::Incomplete { .. } => (),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn three_dates() {
        match "2021-09-09T20:00:00/2021-09-09T21:00:00/2021-09-09T22:00:00"
            .parse::<Slot>()
            .unwrap_err()
        {
            ParseSlotError::TooMuch { .. } => (),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn blank() {
        let slot = "".parse::<Slot>().unwrap_err();
        assert_eq!(slot, ParseSlotError::Blank)
    }
}
