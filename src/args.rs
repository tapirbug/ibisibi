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
    /// indexes of the destinations to loop through, in range 0-999.
    #[argh(positional)]
    pub indexes: Vec<usize>,
    /// interval to wait before switching to the next destination.
    #[argh(option, short = 'i', default = "5.0")]
    pub interval_secs: f64,
    /// serial port to use, e.g. /dev/ttyUSB0 on Linux, or COM5 on Windows.
    #[argh(option, short = 's')]
    pub serial: String,
    /// first destination to show and start of the cycle
    #[argh(option, short = 'f', default = "0")]
    pub from: usize,
    /// last destination to show and start of the cycle
    #[argh(option, short = 't', default = "0")]
    pub to: usize,
}
