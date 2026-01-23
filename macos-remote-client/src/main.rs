use anyhow::Result;
use clap::{Parser, Subcommand};
use macos_remote_protocol::pb::{
    ExecRequest, NotifyRequest, OpReadRequest,
    mac_os_remote_service_client::MacOsRemoteServiceClient,
};
use tonic::transport::Channel;

#[derive(Parser, Debug)]
#[command(name = "macos-remote")]
#[command(about = "CLI client for macOS remote control")]
struct Args {
    /// Server address
    #[arg(long, default_value = "http://127.0.0.1:50052")]
    addr: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Send a notification
    Notify {
        /// Notification title
        title: String,
        /// Notification message
        message: String,
        /// Sound name (e.g., Ping, Pop, Glass, default) or file path
        #[arg(short, long)]
        sound: Option<String>,
    },
    /// Read a secret from 1Password
    OpRead {
        /// Secret reference (e.g., op://vault/item/field)
        reference: String,
    },
    /// Execute a command on the server
    Exec {
        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

async fn connect(addr: &str) -> Result<MacOsRemoteServiceClient<Channel>> {
    let client = MacOsRemoteServiceClient::connect(addr.to_string()).await?;
    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut client = connect(&args.addr).await?;

    match args.command {
        Command::Notify {
            title,
            message,
            sound,
        } => {
            let response = client
                .notify(NotifyRequest {
                    title,
                    message,
                    sound: sound.unwrap_or_default(),
                })
                .await?
                .into_inner();

            if response.success {
                println!("Notification sent successfully");
            } else {
                eprintln!("Notification failed: {}", response.error);
                std::process::exit(1);
            }
        }
        Command::OpRead { reference } => {
            let response = client
                .op_read(OpReadRequest { reference })
                .await?
                .into_inner();

            println!("{}", response.value);
        }
        Command::Exec { args } => {
            let response = client.exec(ExecRequest { args }).await?.into_inner();

            if !response.stdout.is_empty() {
                println!("{}", response.stdout);
            }
            if !response.stderr.is_empty() {
                eprintln!("{}", response.stderr);
            }
            std::process::exit(response.exit_code);
        }
    }

    Ok(())
}
