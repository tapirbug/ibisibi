mod args;
mod cycle;
mod destination;
mod list;
mod plan;
mod range;
mod slot;
mod telegram;

fn main() -> Result<(), String> {
    let args: args::TopLevel = argh::from_env();
    match args.invocation {
        args::Invocation::List(list) => list::list(list).map_err(|e| format!("{}", e)),
        args::Invocation::Destination(destination) => {
            destination::destination(&destination).map_err(|e| format!("{}", e))
        }
        args::Invocation::Cycle(cycle) => cycle::cycle(&cycle).map_err(|e| format!("{}", e)),
    }
}
