mod args;
mod cycle;
mod destination;
mod list;
mod parity;
mod plan;
mod range;
mod record;
mod run;
mod slot;
mod status;
mod telegram;
mod serial;

fn main() -> Result<(), String> {
    let args: args::TopLevel = argh::from_env();
    run::run(args.invocation)
}
