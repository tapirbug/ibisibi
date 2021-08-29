use crate::range::{ParseRangeError, Range};
use crate::slot::{ParseSlotError, Slot};
use std::str::FromStr;
use thiserror::Error;

/// A range with an optinal associated time range.
#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    range: Range,
    slot: Option<Slot>,
}

impl Plan {
    pub fn range(&self) -> Range {
        self.range
    }

    pub fn slot(&self) -> Option<Slot> {
        self.slot
    }
}

impl FromStr for Plan {
    type Err = ParsePlanError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.is_empty() {
            return Err(ParsePlanError::Blank);
        }

        let mut tokens = source.split('@');
        let range: Range = tokens.next().unwrap().parse()?; // unwrap is safe because we checked for empty above
        let slot = match tokens.next() {
            Some(scheduled_slot) => {
                let slot: Slot = scheduled_slot.parse()?;
                Some(slot)
            }
            None => None,
        };

        if tokens.next().is_some() {
            return Err(ParsePlanError::too_much(source));
        }

        Ok(Plan { range, slot })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParsePlanError {
    #[error("Could not parse blank string as a number or number range")]
    Blank,
    #[error("Number or number range contains more than two scheduled times: `{input}`")]
    TooMuch { input: String },
    #[error("{0}")]
    ParseRange(#[from] ParseRangeError),
    #[error("{0}")]
    ParseSlot(#[from] ParseSlotError),
}

impl ParsePlanError {
    fn too_much(source: &str) -> Self {
        Self::TooMuch {
            input: source.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_with_slot() {
        let input: Plan = "0-10@2020-01-01T00:00:00/2020-01-01T00:00:00"
            .parse()
            .unwrap();
        assert_eq!(
            input,
            Plan {
                range: "0-10".parse().unwrap(),
                slot: Some("2020-01-01T00:00:00/2020-01-01T00:00:00".parse().unwrap())
            }
        )
    }

    #[test]
    fn parse_without_slot() {
        let input: Plan = "0".parse().unwrap();
        assert_eq!(
            input,
            Plan {
                range: "0".parse().unwrap(),
                slot: None
            }
        )
    }

    #[test]
    fn blank() {
        let plan_error = "".parse::<Plan>().unwrap_err();
        assert_eq!(plan_error, ParsePlanError::Blank)
    }

    #[test]
    fn too_much() {
        let input =
            "0@2020-01-01T00:00:00/2020-01-01T00:00:00@2020-01-01T00:00:00/2020-01-01T00:00:00";
        let plan_error = input.parse::<Plan>().unwrap_err();
        assert_eq!(
            plan_error,
            ParsePlanError::TooMuch {
                input: input.to_string()
            }
        )
    }

    #[test]
    fn malformed_range() {
        let input = "0--9@2020-01-01T00:00:00/2020-01-01T00:00:00";
        match input.parse::<Plan>().unwrap_err() {
            ParsePlanError::ParseRange(_) => (),
            error => panic!("Unexpected error: {:?}", error),
        }
    }

    #[test]
    fn malformed_slot() {
        let input = "0-10@2020-01-01T00:00:00//2020-01-01T00:00:00";
        match input.parse::<Plan>().unwrap_err() {
            ParsePlanError::ParseSlot(_) => (),
            error => panic!("Unexpected error: {:?}", error),
        }
    }
}
