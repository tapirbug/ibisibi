use crate::plan::Plan;
use argh::FromArgs;

/// Write IBIS telegrams to serial ports or list available serial ports.
#[derive(FromArgs)]
pub struct TopLevel {
    #[argh(subcommand)]
    pub invocation: Invocation,
}

/// Inner top-level command.
#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Invocation {
    List(List),
    Destination(Destination),
    Cycle(Cycle),
}

/// List available serial ports.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {}

/// Set the currently shown destination to the one with the given index
/// using telegram DS003.
#[derive(FromArgs)]
#[argh(subcommand, name = "destination")]
pub struct Destination {
    /// index of the destination to set, in range 0-999.
    #[argh(positional)]
    pub index: u16,
    /// optional line number, in range 1-999.
    #[argh(option, short = 'l')]
    pub line: Option<u16>,
    /// serial port to use, e.g. /dev/ttyUSB0 on Linux, or COM5 on Windows.
    #[argh(option, short = 's')]
    pub serial: String,
}

/// Loop through the given destination indexes in regular intervals.
///
/// When from/to and positional indexes are both used, then will start
/// with directly specified indexes, then from/to, and then over again.
#[derive(FromArgs)]
#[argh(subcommand, name = "cycle")]
pub struct Cycle {
    /// indexes or index ranges of the destinations to loop through, with optional scheduled time e.g. 8 or 0-5@2021-06-03T00:00:00.
    ///
    /// Indexes must be in range 0 to 999.
    #[argh(positional)]
    pub indexes: Vec<Plan>,
    /// interval to wait before switching to the next destination.
    #[argh(option, short = 'i', default = "5.0")]
    pub interval_secs: f64,
    /// show scheduled destinations this many hours before scheduled start
    #[argh(option, short = 'i', default = "12")]
    pub lookahead: i64,
    /// serial port to use, e.g. /dev/ttyUSB0 on Linux, or COM5 on Windows.
    #[argh(option, short = 's')]
    pub serial: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn number_ranges() {
        let expected_serial = "/dev/ttyUSB0";
        let args = ["cycle", "0", "0-5", "-s", expected_serial];
        let args: TopLevel = argh::FromArgs::from_args(&["ibisibi"], &args).unwrap();
        match args.invocation {
            Invocation::Cycle(Cycle {
                indexes, serial, ..
            }) => {
                assert_eq!(indexes, vec!["0".parse().unwrap(), "0-5".parse().unwrap()]);
                assert_eq!(serial, expected_serial);
            }
            _ => panic!("unexpected subcommand"),
        }
    }

    #[test]
    fn time_ranges() {
        let args = [
            "cycle",
            "0",
            "2-3@2021-09-09T20:00:00/2021-09-09T21:00:00",
            "--serial",
            "/dev/ttyUSB0",
            "--interval-secs",
            "8",
        ];
        let args: TopLevel = argh::FromArgs::from_args(&["ibisibi"], &args).unwrap();
        match args.invocation {
            Invocation::Cycle(Cycle { indexes, .. }) => {
                assert_eq!(indexes[0].range(), "0".parse().unwrap());
                assert!(indexes[0].slot().is_none());
                assert_eq!(indexes[1].range(), "2-3".parse().unwrap());
                assert_eq!(
                    indexes[1].slot(),
                    Some("2021-09-09T20:00:00/2021-09-09T21:00:00".parse().unwrap())
                );
                assert_eq!(indexes.len(), 2)
            }
            _ => panic!("unexpected subcommand"),
        }
    }
}
