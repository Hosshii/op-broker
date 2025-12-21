use anyhow::Result;
use clap::{Parser, Subcommand};
use protocol::SecretId;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Parser)]
#[command(name = "op-brokerctl", about = "1Password broker client CLI")]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "SOCKET",
        default_value = "/run/op-broker.sock"
    )]
    socket: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Read { id: SecretId },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;
    let cli = Cli::parse();
    run(cli).await
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Read { id } => {
            info!(socket = %cli.socket.display(), %id, "read command requested (not yet implemented)");
        }
    }
    Ok(())
}

fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env_filter).try_init()?;
    Ok(())
}
