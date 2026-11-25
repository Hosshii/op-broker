use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Local command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Use the macOS remote broker
    OpBroker(op_broker::Args),
}

fn main() -> Result<ExitCode> {
    match Cli::parse().command {
        Command::OpBroker(args) => Ok(ExitCode::from(op_broker::run(args)?)),
    }
}
