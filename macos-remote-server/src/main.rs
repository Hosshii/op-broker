mod exec_client;
mod notify;
mod op_client;
mod service;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use macos_remote_protocol::pb::mac_os_remote_service_server::MacOsRemoteServiceServer;
use service::MacOsRemoteServiceImpl;
use tonic::transport::Server;
use tracing::info;

use crate::exec_client::ExecClient;

#[derive(Parser, Debug)]
#[command(name = "macos-remote-server")]
#[command(about = "gRPC server for macOS remote control")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "50052")]
    port: u16,

    /// Path to executable for exec command
    #[arg(long)]
    exec_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let addr = format!("127.0.0.1:{}", args.port).parse()?;

    let exec_client = args
        .exec_path
        .map(|p| ExecClient::from_path(p.clone()))
        .transpose()?;

    let service = MacOsRemoteServiceImpl::new(exec_client);

    info!("Starting macOS remote server on {}", addr);

    Server::builder()
        .add_service(MacOsRemoteServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
