use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

mod args;
mod cycle;
mod destination;
mod devices;
mod flash;
mod list;
mod parity;
mod plan;
mod range;
mod record;
mod run;
mod scan;
mod serial;
mod slot;
mod status;
mod telegram;

fn main() -> Result<(), String> {
    // a builder for `FmtSubscriber`.
    let stderr = std::io::stderr
        // all spans/events including TRACE will be written to stderr.
        .with_max_level(Level::TRACE);
    tracing_subscriber::fmt().with_writer(stderr).init();

    let args: args::TopLevel = argh::from_env();
    run::run(args.invocation)
}
