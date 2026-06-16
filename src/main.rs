use clap::Parser;
use ncal::{app, cli::Cli, config};

fn main() {
    let cli = Cli::parse();
    match config::Config::from_cli(&cli) {
        Ok(config) => {
            if let Err(e) = config.validate() {
                eprintln!("Error: {e}");
            } else if let Err(e) = app::run(config) {
                eprintln!("Error: {e}");
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
