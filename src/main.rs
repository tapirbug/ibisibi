mod args;
mod cycle;
mod destination;
mod devices;
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
    let args: args::TopLevel = argh::from_env();
    run::run(args.invocation)
}
