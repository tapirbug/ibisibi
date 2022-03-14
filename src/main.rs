#![feature(backtrace)]

use tracing::Level;

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
    std::env::set_var("RUST_BACKTRACE", "1"); // always enable backtraces

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_writer(std::io::stderr)
        .init();

    let args: args::TopLevel = argh::from_env();
    run::run(args.invocation)
}
