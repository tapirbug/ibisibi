use tracing::Level;
use tracing_subscriber::FmtSubscriber;

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
mod slot;
mod status;
mod telegram;
mod serial;

fn main() -> Result<(), String> {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let args: args::TopLevel = argh::from_env();
    run::run(args.invocation)
}
