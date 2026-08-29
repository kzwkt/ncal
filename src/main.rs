use std::process::ExitCode;

use clap::Parser;
use ncal::{app, cli::Cli, config};

fn main() -> ExitCode {
    let cli = Cli::parse();

    // A non-zero exit matters to callers that shell out to ncal and need to
    // tell "no festivals today" apart from "that invocation failed".
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), ncal::NcalError> {
    let config = config::Config::from_cli(cli)?;
    config.validate()?;
    app::run(config)
}
