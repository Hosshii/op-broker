mod config;
mod service;

use crate::{config::BrokerConfig, service::BrokerRpcService};
use anyhow::{Context, Result};
use clap::Parser;
use protocol::pb::broker_service_server::BrokerServiceServer;
use std::{fs, path::Path, path::PathBuf, sync::Arc};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

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
    let config = Arc::new(BrokerConfig::load(&cli.config).context("failed to load configuration")?);
    prepare_socket_path(&config.socket_path).context("failed to prepare socket path")?;
    let uds = UnixListener::bind(&config.socket_path)
        .with_context(|| format!("failed to bind socket at {}", config.socket_path.display()))?;
    let incoming = UnixListenerStream::new(uds);
    info!(
        socket = %config.socket_path.display(),
        items = config.item_count(),
        "listening for gRPC over Unix socket"
    );

    let service = BrokerRpcService::new(config);
    Server::builder()
        .add_service(BrokerServiceServer::new(service))
        .serve_with_incoming(incoming)
        .await
        .context("gRPC server failed")?;
    Ok(())
}

fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(env_filter)
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

fn prepare_socket_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("failed to remove existing socket {}", path.display()))?;
    }
    Ok(())
}
