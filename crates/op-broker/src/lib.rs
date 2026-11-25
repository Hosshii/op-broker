mod client;
mod server;

use anyhow::Result;
use clap::Subcommand;

#[derive(clap::Args)]
pub struct Args {
    /// gRPC server address for client commands
    #[arg(long, default_value = "http://127.0.0.1:50052")]
    addr: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the gRPC server on macOS
    Serve(server::Args),
    #[command(flatten)]
    Client(client::Command),
}

#[tokio::main]
pub async fn run(args: Args) -> Result<u8> {
    match args.command {
        Command::Serve(args) => {
            server::serve(args).await?;
            Ok(0)
        }
        Command::Client(command) => client::run(command, &args.addr).await,
    }
}
