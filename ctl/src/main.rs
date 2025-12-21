use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use hyper_util::rt::TokioIo;
use protocol::{SecretId, pb::ReadSecretRequest, pb::broker_service_client::BrokerServiceClient};
use serde_json::json;
use std::{path::PathBuf, process, sync::Arc};
use tokio::net::UnixStream;
use tonic::{
    Request, Status,
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
    Read(ReadArgs),
}

#[derive(Debug, Args, Clone)]
struct ReadArgs {
    #[arg(value_name = "ID")]
    id: SecretId,
    #[arg(long, value_name = "TEXT", help = "Nonce 文字列を broker に渡す")]
    nonce: Option<String>,
    #[arg(long, help = "JSON 形式 ({\"ok\":true,...}) で出力する")]
    json: bool,
    #[arg(long, help = "stdout を抑制 (JSON モードでは無視)")]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;
    let cli = Cli::parse();
    if let Err(err) = run(cli).await {
        eprintln!("error: {err}");
        process::exit(1);
    }
    Ok(())
}

async fn run(cli: Cli) -> Result<()> {
    let mut client = connect_via_uds(cli.socket.clone()).await?;
    info!(socket = %cli.socket.display(), "connected to broker");

    match cli.command {
        Command::Read(args) => handle_read(&mut client, args).await,
    }
}

fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(env_filter)
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

async fn handle_read(client: &mut BrokerServiceClient<Channel>, args: ReadArgs) -> Result<()> {
    let request = Request::new(ReadSecretRequest {
        id: args.id.into_string(),
        nonce: args.nonce.clone().unwrap_or_default(),
    });
    match client.read_secret(request).await {
        Ok(response) => {
            let reply = response.into_inner();
            if args.json {
                println!("{}", json!({"ok": true, "value": reply.value}));
            } else if !args.quiet {
                println!("{}", reply.value);
            }
            Ok(())
        }
        Err(status) => emit_error(&status, args.json),
    }
}

fn emit_error(status: &Status, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            json!({"ok": false, "code": status.code().to_string(), "message": status.message()})
        );
    }
    Err(anyhow::anyhow!(status.to_string()))
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
