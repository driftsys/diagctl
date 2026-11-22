mod checks;
mod cli;
mod color;
mod commands;
mod contract;
mod geom;
mod svg;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { target, file } => commands::check::run(&file, target),
        Command::Optimize { file } => commands::optimize::run(&file),
        Command::Freshness { file } => commands::freshness::run(&file),
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("diagctl: {err:#}");
            std::process::exit(2);
        }
    }
}
