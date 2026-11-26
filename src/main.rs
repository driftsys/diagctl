mod checks;
mod cli;
mod color;
mod commands;
mod contract;
mod geom;
mod output;
mod svg;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check {
            target,
            json,
            files,
        } => commands::check::run(&files, target, json),
        Command::Optimize { json, file } => commands::optimize::run(&file, json),
        Command::Freshness { json, file } => commands::freshness::run(&file, json),
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("diagctl: {err:#}");
            std::process::exit(2);
        }
    }
}
