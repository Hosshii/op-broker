#[cfg(target_os = "linux")]
mod cli {
    use anyhow::{Context, Result};
    use clap::Parser;
    use op_secrets_fs::{RunConfig, SecretEntry, run};
    use serde::Deserialize;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::Duration;

    /// op-secrets-fs CLI の引数定義。
    #[derive(Debug, Parser)]
    #[command(name = "op-secrets-fs", about = "FUSE filesystem backed by op-broker")]
    struct Args {
        /// Where to mount the read-only filesystem
        #[arg(long, default_value = "/run/op-secrets", value_name = "PATH")]
        mountpoint: PathBuf,

        /// Path to the Unix domain socket served by op-broker
        #[arg(long, value_name = "PATH")]
        socket: PathBuf,

        /// JSON file describing path/secret_reference mappings
        #[arg(long, value_name = "FILE")]
        config: PathBuf,

        /// RPC timeout in seconds
        #[arg(long, default_value_t = 30, value_name = "SECONDS")]
        timeout: u64,
    }

    #[derive(Debug, Deserialize)]
    struct ConfigFile {
        entries: Vec<ConfigEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct ConfigEntry {
        path: PathBuf,
        secret_reference: String,
    }

    pub fn main() {
        if let Err(err) = run_cli() {
            eprintln!("op-secrets-fs failed: {err:?}");
            process::exit(1);
        }
    }

    fn run_cli() -> Result<()> {
        let args = Args::parse();
        let entries = load_config(&args.config)?;
        let config = RunConfig::try_new(
            args.mountpoint,
            args.socket,
            entries,
            Duration::from_secs(args.timeout),
        )?;
        run(config)
    }

    fn load_config(path: &Path) -> Result<Vec<SecretEntry>> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let cfg: ConfigFile = serde_json::from_str(&data)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        let mut entries = Vec::new();
        for (idx, entry) in cfg.entries.into_iter().enumerate() {
            let secret_entry = SecretEntry::try_new(entry.secret_reference, entry.path)
                .with_context(|| format!("entry #{idx} has invalid secret_reference"))?;
            entries.push(secret_entry);
        }
        Ok(entries)
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("op-secrets-fs is only supported on Linux hosts");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() {
    cli::main();
}
