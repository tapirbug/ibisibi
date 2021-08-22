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
}

/// List available serial ports.
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {}
