mod args;
mod list;
mod telegram;

fn main() -> Result<(), String> {
    let args: args::TopLevel = argh::from_env();
    match args.invocation {
        args::Invocation::List(list) => list::list(list).map_err(|e| format!("{}", e)),
    }
}
