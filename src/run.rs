use crate::args::{Invocation, Run};
use serde_yaml::from_reader;
use std::fs::File;
use thiserror::Error;

pub fn run(invocation: Invocation) -> Result<(), String> {
    match invocation {
        Invocation::Run(run) => run_yaml(run).map_err(|e| format!("{}", e)),
        Invocation::List(list) => crate::list::list(list).map_err(|e| format!("{}", e)),
        Invocation::Scan(scan) => crate::devices::scan(scan).map_err(|e| format!("{}", e)),
        Invocation::Destination(destination) => {
            crate::destination::destination(&destination).map_err(|e| format!("{}", e))
        }
        Invocation::Cycle(cycle) => crate::cycle::cycle(&cycle).map_err(|e| format!("{}", e)),
        Invocation::Flash(flash) => crate::flash::flash(flash).map_err(|e| format!("{}", e))
    }
}

fn run_yaml(opts: Run) -> Result<(), RunError> {
    let file = File::open(opts.config)?;
    let invocation = from_reader(file)?;
    run(invocation).map_err(RunError::Cmd)
}

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Could not open specified YAML configuration file: {0}")]
    IO(#[from] std::io::Error),
    #[error("Could not parse specified YAML configuration file: {0}")]
    Deserialize(#[from] serde_yaml::Error),
    #[error("{0}")]
    Cmd(String),
}
