use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hyper_util::rt::TokioIo;
use protocol::{SecretId, pb::ReadSecretRequest, pb::broker_service_client::BrokerServiceClient};
use std::{path::PathBuf, sync::Arc};
use tokio::net::UnixStream;
use tonic::{
    Request,
    transport::{Channel, Endpoint},
};
use tower::service_fn;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

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
    let mut client = connect_via_uds(cli.socket.clone()).await?;
    info!(socket = %cli.socket.display(), "connected to broker");

    match cli.command {
        Command::Read { id } => {
            let request = Request::new(ReadSecretRequest {
                id: id.into_string(),
                nonce: String::new(),
            });
            let response = client.read_secret(request).await?;
            let reply = response.into_inner();
            println!("{}", reply.value);
        }
    }
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

async fn connect_via_uds(path: PathBuf) -> Result<BrokerServiceClient<Channel>> {
    let endpoint = Endpoint::from_static("http://[::]:50051");
    let path = Arc::new(path);
    let channel = endpoint
        .connect_with_connector(service_fn(move |_| {
            let path = path.clone();
            async move {
                let stream = UnixStream::connect(path.as_ref()).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .context("failed to connect over unix socket")?;
    Ok(BrokerServiceClient::new(channel))
}
