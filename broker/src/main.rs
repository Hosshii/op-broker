mod config;

use crate::config::BrokerConfig;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Parser)]
#[command(name = "op-brokerd", about = "1Password broker daemon")]
struct Cli {
    #[arg(long, value_name = "FILE", help = "Path to config.json")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;
    let cli = Cli::parse();
    run(cli).await
}

async fn run(cli: Cli) -> Result<()> {
    let config = BrokerConfig::load(&cli.config).context("failed to load configuration")?;
    info!(
        socket = %config.socket_path.display(),
        items = config.item_count(),
        "configuration loaded"
    );
    info!("waiting for Ctrl+C to exit (server not yet implemented)");
    signal::ctrl_c()
        .await
        .context("failed to listen for shutdown signal")?;
    Ok(())
}

fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env_filter).try_init()?;
    Ok(())
}
