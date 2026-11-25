mod exec_client;
mod notify;
mod op_client;
mod service;

use std::path::PathBuf;

use anyhow::Result;
use exec_client::ExecClient;
use macos_remote_protocol::pb::mac_os_remote_service_server::MacOsRemoteServiceServer;
use service::MacOsRemoteServiceImpl;
use tonic::transport::Server;
use tracing::info;

#[derive(clap::Args, Debug)]
pub(crate) struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "50052")]
    port: u16,

    /// Path to executable for exec command
    #[arg(long)]
    exec_path: Option<PathBuf>,
}

pub(crate) async fn serve(args: Args) -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = format!("127.0.0.1:{}", args.port).parse()?;

    let exec_client = args.exec_path.map(ExecClient::from_path).transpose()?;

    let service = MacOsRemoteServiceImpl::new(exec_client);

    info!("Starting macOS remote server on {}", addr);

    Server::builder()
        .add_service(MacOsRemoteServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
