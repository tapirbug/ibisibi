mod args;
mod cycle;
mod destination;
mod list;
mod plan;
mod range;
mod run;
mod slot;
mod telegram;

fn main() -> Result<(), String> {
    let args: args::TopLevel = argh::from_env();
    run::run(args.invocation)
}
