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
