//! Inclusive index range that supports both start < end (forward iteration) and
//! start > end (backward iteration).
//!
//! Can be parsed from strings like `0-10` but also single numbers like `4`.
//! The notation is inclusive for both the start and the end element.

use std::cmp::Ordering;
use std::iter::Iterator;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Range {
    from: usize,
    to: usize,
}

#[derive(Debug, Clone)]
pub struct RangeIter {
    range: Range,
    exhausted: bool,
}

impl Range {
    pub fn iter(&self) -> RangeIter {
        RangeIter {
            range: *self,
            exhausted: false,
        }
    }
}

impl Iterator for RangeIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        match self.range.from.cmp(&self.range.to) {
            Ordering::Equal => {
                self.exhausted = true;
                Some(self.range.from)
            }
            Ordering::Less => {
                let next = self.range.from;
                self.range.from = self.range.from.saturating_add(1);
                Some(next)
            }
            Ordering::Greater => {
                let next = self.range.from;
                self.range.from = self.range.from.saturating_sub(1);
                Some(next)
            }
        }
    }
}

impl FromStr for Range {
    type Err = ParseRangeError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.is_empty() {
            return Err(ParseRangeError::Blank);
        }

        if source == "-" {
            return Err(ParseRangeError::malformed(source));
        }

        let mut numbers = source.split('-');
        let first = if let Some(first) = numbers.next() {
            parse_num_or_zero_when_empty(first)?
        } else {
            return Err(ParseRangeError::malformed(source));
        };

        let second = if let Some(second) = numbers.next() {
            parse_num_or_zero_when_empty(second)?
        } else {
            first
        };

        if let Some(_superfluous) = numbers.next() {
            return Err(ParseRangeError::malformed(source));
        }

        Ok(Range {
            from: first,
            to: second,
        })
    }
}

fn parse_num_or_zero_when_empty(source: &str) -> Result<usize, ParseRangeError> {
    Ok(if source.is_empty() {
        0
    } else {
        source.parse::<usize>()?
    })
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseRangeError {
    #[error("Could not parse `{offending_input}` as a number or number range")]
    Malformed { offending_input: String },
    #[error("Could not parse `{0}` as a number")]
    NumberFormat(#[from] std::num::ParseIntError),
    #[error("Could not parse blank string as a range")]
    Blank,
}

impl ParseRangeError {
    fn malformed(source: &str) -> Self {
        Self::Malformed {
            offending_input: source.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_single_num() {
        let range: Range = "0".parse().unwrap();
        assert_eq!(range, Range { from: 0, to: 0 })
    }

    #[test]
    fn parse_single_elem_range() {
        let range: Range = "0-0".parse().unwrap();
        assert_eq!(range, Range { from: 0, to: 0 })
    }

    #[test]
    fn parse_forward_range() {
        let range: Range = "10-100".parse().unwrap();
        assert_eq!(range, Range { from: 10, to: 100 })
    }

    #[test]
    fn parse_backward_range() {
        let range: Range = "100-10".parse().unwrap();
        assert_eq!(range, Range { from: 100, to: 10 })
    }

    #[test]
    fn parse_missing_start() {
        let range: Range = "-10".parse().unwrap();
        assert_eq!(range, Range { from: 0, to: 10 })
    }

    #[test]
    fn parse_missing_end() {
        let range: Range = "10-".parse().unwrap();
        assert_eq!(range, Range { from: 10, to: 0 })
    }

    #[test]
    fn parse_with_whitespace_both_sides() {
        let source = "10 - 10";
        match source.parse::<Range>() {
            Err(ParseRangeError::NumberFormat(_)) => (),
            other => panic!(
                "parse unexpectedly succeeded or had unexpected error type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn parse_with_whitespace_start() {
        let source = "10- 10";
        match source.parse::<Range>() {
            Err(ParseRangeError::NumberFormat(_)) => (),
            other => panic!(
                "parse unexpectedly succeeded or had unexpected error type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn parse_double_dash() {
        let source = "10--10";
        match source.parse::<Range>() {
            Err(ParseRangeError::Malformed { offending_input }) => {
                assert_eq!(offending_input, source)
            }
            other => panic!(
                "parse unexpectedly succeeded or had unexpected error type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn parse_only_dash() {
        let source = "-";
        match source.parse::<Range>() {
            Err(ParseRangeError::Malformed { offending_input }) => {
                assert_eq!(offending_input, source)
            }
            other => panic!(
                "parse unexpectedly succeeded or had unexpected error type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn parse_only_dashes() {
        let source = "--";
        match source.parse::<Range>() {
            Err(ParseRangeError::Malformed { offending_input }) => {
                assert_eq!(offending_input, source)
            }
            other => panic!(
                "parse unexpectedly succeeded or had unexpected error type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn parse_empty() {
        let source = "";
        match source.parse::<Range>() {
            Err(ParseRangeError::Blank) => (),
            other => panic!(
                "parse unexpectedly succeeded or had unexpected error type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn iterate_single_elem() {
        let range: Vec<usize> = Range { from: 0, to: 0 }.iter().collect();
        assert_eq!(range, vec![0])
    }

    #[test]
    fn iterate_forward() {
        let range: Vec<usize> = Range { from: 5, to: 7 }.iter().collect();
        assert_eq!(range, vec![5, 6, 7])
    }

    #[test]
    fn iterate_backward() {
        let range: Vec<usize> = Range { from: 2, to: 0 }.iter().collect();
        assert_eq!(range, vec![2, 1, 0])
    }

    #[test]
    fn iterate_flattened_cycled_vec() {
        let vec = vec![
            "0-1".parse::<Range>().unwrap(),
            "0".parse::<Range>().unwrap(),
        ];
        let flattened = vec
            .iter()
            .flat_map(|r| r.iter())
            .cycle()
            .take(5)
            .collect::<Vec<_>>();
        assert_eq!(flattened, vec![0, 1, 0, 0, 1])
    }
}
