use self::filesystem::OpSecretsFs;
use crate::client::OpBrokerClient;
use anyhow::{Context, Result, anyhow};
use fuser::MountOption;
use nix::mount::{MntFlags, MsFlags, mount, umount2};
use protocol::OpSecretReference;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Builder;

pub mod filesystem;

#[derive(Debug, Clone)]
pub struct SecretEntry {
    reference: OpSecretReference,
    bind_target: PathBuf,
}

pub struct RunConfig {
    mountpoint: PathBuf,
    socket: PathBuf,
    entries: Vec<SecretEntry>,
    timeout: Duration,
}

struct BindEntry {
    name: String,
    bind_target: PathBuf,
}

impl SecretEntry {
    pub fn try_new(reference: String, bind_target: PathBuf) -> Result<Self> {
        let reference = OpSecretReference::parse(&reference)?;
        if !bind_target.is_absolute() {
            anyhow::bail!("bind target {} must be absolute", bind_target.display());
        }
        Ok(Self {
            reference,
            bind_target,
        })
    }
}

impl RunConfig {
    pub fn try_new(
        mountpoint: PathBuf,
        socket: PathBuf,
        entries: Vec<SecretEntry>,
        timeout: Duration,
    ) -> Result<Self> {
        if !mountpoint.exists() {
            anyhow::bail!("mountpoint {} does not exist", mountpoint.display());
        }
        if entries.is_empty() {
            anyhow::bail!("config must include at least one entry");
        }
        Ok(Self {
            mountpoint,
            socket,
            entries,
            timeout,
        })
    }
}

/// Linux 環境で FUSE を起動する。
pub fn run(config: RunConfig) -> Result<()> {
    let runtime = Arc::new(
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("failed to initialize tokio runtime")?,
    );
    let client = runtime
        .block_on(OpBrokerClient::connect(config.socket))
        .context("failed to initialize broker client")?;
    let mount_entries = config.entries.clone();
    let bind_entries = config.entries.clone();
    let fs = OpSecretsFs::new(runtime.clone(), client, mount_entries, config.timeout);
    let bind_entries: Vec<BindEntry> = bind_entries
        .into_iter()
        .map(|entry| BindEntry {
            name: fs.name_for_reference(&entry.reference),
            bind_target: entry.bind_target,
        })
        .collect();

    let mount_opts = vec![
        MountOption::RO,
        MountOption::FSName("op-secrets".into()),
        MountOption::AutoUnmount,
    ];
    let mountpoint = config.mountpoint.clone();
    let fuse_thread = thread::spawn(move || {
        fuser::mount2(fs, &mountpoint, &mount_opts).context("failed to mount op-secrets fs")
    });

    wait_for_sources(&config.mountpoint, &bind_entries)?;
    let bound = setup_bind_mounts(&config.mountpoint, &bind_entries)?;

    fuse_thread
        .join()
        .map_err(|_| anyhow!("fuse thread panicked"))??;

    cleanup_bind_mounts(bound)?;
    Ok(())
}

fn wait_for_sources(mountpoint: &Path, entries: &[BindEntry]) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut ready = true;
        for entry in entries {
            let source = mountpoint.join(&entry.name);
            if !source.exists() {
                ready = false;
                break;
            }
        }
        if ready {
            return Ok(());
        }
        if Instant::now() > deadline {
            anyhow::bail!("failed to see files exposed under {}", mountpoint.display());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn setup_bind_mounts(mountpoint: &Path, entries: &[BindEntry]) -> Result<Vec<PathBuf>> {
    let mut bound = Vec::new();
    for entry in entries {
        let source = mountpoint.join(&entry.name);
        let target = &entry.bind_target;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        if target.exists() {
            anyhow::bail!(
                "bind target {} already exists; remove it before starting",
                target.display()
            );
        } else {
            fs::File::create(target)
                .with_context(|| format!("failed to create {}", target.display()))?;
        }
        mount(
            Some(source.as_path()),
            target,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .with_context(|| format!("failed to bind mount {}", target.display()))?;
        mount::<Path, Path, str, str>(
            None,
            target,
            None,
            MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY,
            None,
        )
        .with_context(|| format!("failed to remount {} read-only", target.display()))?;
        bound.push(target.clone());
    }
    Ok(bound)
}

fn cleanup_bind_mounts(bound: Vec<PathBuf>) -> Result<()> {
    for target in bound.into_iter().rev() {
        let _ = umount2(&target, MntFlags::MNT_DETACH);
        let _ = fs::remove_file(&target);
    }
    Ok(())
}
