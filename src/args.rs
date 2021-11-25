use crate::plan::Plan;
use argh::FromArgs;
use serde::Deserialize;
use std::path::PathBuf;

/// Write IBIS telegrams to serial ports or list available serial ports.
#[derive(FromArgs)]
pub struct TopLevel {
    #[argh(subcommand)]
    pub invocation: Invocation,
}

/// Inner top-level command.
#[derive(FromArgs, Deserialize)]
#[argh(subcommand)]
#[serde(rename_all = "snake_case")]
pub enum Invocation {
    #[serde(skip)]
    Run(Run),
    List(List),
    Flash(Flash),
    #[serde(skip)]
    Scan(Scan),
    Destination(Destination),
    Cycle(Cycle),
}

/// Take run parameters from a specified YAML configuration file.
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
pub struct Run {
    #[argh(positional)]
    pub config: PathBuf,
}

/// List available serial ports.
#[derive(FromArgs, Deserialize)]
#[argh(subcommand, name = "list")]
pub struct List {}

/// Finds available addresses of display devices on the specified serial port.
#[derive(FromArgs)]
#[argh(subcommand, name = "scan")]
pub struct Scan {
    /// serial port to use, e.g. /dev/ttyUSB0 on Linux, or COM5 on Windows.
    #[argh(option, short = 's')]
    pub serial: String,
}

/// Set the currently shown destination to the one with the given index
/// using telegram DS003.
#[derive(FromArgs, Deserialize)]
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

/// Flash a new sign database in .hex format to a BS210 sign.
#[derive(FromArgs, Deserialize, Debug)]
#[argh(subcommand, name = "flash")]
pub struct Flash {
    /// path to a BS210-compatible sign database in `.hex` format.
    #[argh(positional)]
    pub sign_db_hex: PathBuf,
    /// IBIS address to flash to in range 0..15.
    #[argh(option, short = 'a')]
    pub address: u8,
    /// serial port to use, e.g. /dev/ttyUSB0 on Linux, or COM5 on Windows.
    #[argh(option, short = 's')]
    pub serial: String,
}

/// Loop through the given destination indexes in regular intervals.
///
/// When from/to and positional indexes are both used, then will start
/// with directly specified indexes, then from/to, and then over again.
#[derive(FromArgs, Deserialize)]
#[argh(subcommand, name = "cycle")]
pub struct Cycle {
    /// indexes or index ranges of the destinations to loop through, with optional scheduled time e.g. 8 or 0-5@2021-06-03T00:00:00.
    ///
    /// Indexes must be in range 0 to 999.
    #[argh(positional)]
    pub plan: Vec<Plan>,
    /// interval to wait before switching to the next destination.
    #[argh(option, short = 'i', default = "5.0")]
    pub interval_secs: f64,
    /// show scheduled destinations this many hours before scheduled start
    #[argh(option, short = 'i', default = "12")]
    pub lookahead: u32,
    /// serial port to use, e.g. /dev/ttyUSB0 on Linux, or COM5 on Windows.
    #[argh(option, short = 's')]
    pub serial: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml::from_str;

    #[test]
    fn number_ranges() {
        let expected_serial = "/dev/ttyUSB0";
        let args = ["cycle", "0", "0-5", "-s", expected_serial];
        let args: TopLevel = argh::FromArgs::from_args(&["ibisibi"], &args).unwrap();
        match args.invocation {
            Invocation::Cycle(Cycle { plan, serial, .. }) => {
                assert_eq!(plan, vec!["0".parse().unwrap(), "0-5".parse().unwrap()]);
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
            Invocation::Cycle(Cycle { plan, .. }) => {
                assert_eq!(plan[0].destinations(), &["0".parse().unwrap()]);
                assert!(plan[0].slots().is_empty());
                assert_eq!(plan[1].destinations(), &["2-3".parse().unwrap()]);
                assert_eq!(
                    plan[1].slots(),
                    &["2021-09-09T20:00:00/2021-09-09T21:00:00".parse().unwrap()]
                );
                assert_eq!(plan.len(), 2)
            }
            _ => panic!("unexpected subcommand"),
        }
    }

    #[test]
    fn deserialize_cycle() {
        let invocation: Invocation =
            from_str(dbg!(include_str!("../examples/cycle.yaml"))).unwrap();
        match invocation {
            Invocation::Cycle(Cycle {
                plan,
                interval_secs,
                lookahead,
                serial,
            }) => {
                assert_eq!(
                    plan,
                    vec! {
                        Plan::range("0"),
                        Plan::range_start_end(
                            "6",
                            "2021-09-09T18:00:00/2021-09-10T00:00:00"
                        )
                    }
                );
                assert_eq!(interval_secs, 9.0);
                assert_eq!(lookahead, 12);
                assert_eq!(serial, "/dev/ttyUSB0");
            }
            _ => panic!("Unexcpected invocation kind"),
        }
    }

    #[test]
    fn deserialize_destination() {
        let invocation: Invocation =
            from_str(include_str!("../examples/destination.yaml")).unwrap();
        match invocation {
            Invocation::Destination(Destination {
                index: 0,
                line: Some(6),
                serial,
            }) => {
                assert_eq!(serial, "COM5");
            }
            _ => panic!("Unexcpected invocation kind"),
        }
    }

    #[test]
    fn deserialize_list() {
        let invocation: Invocation = from_str(include_str!("../examples/list.yaml")).unwrap();
        match invocation {
            Invocation::List(_list) => {}
            _ => panic!("Unexcpected invocation kind"),
        }
    }
}
