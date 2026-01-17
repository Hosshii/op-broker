use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct Notifier {
    binary: PathBuf,
}

impl Notifier {
    pub fn discover() -> Result<Self, NotifyError> {
        let binary =
            which::which("terminal-notifier").map_err(|_| NotifyError::ExecutableNotFound)?;
        Ok(Self { binary })
    }

    pub async fn notify(&self, title: &str, message: &str) -> Result<(), NotifyError> {
        let output = Command::new(&self.binary)
            .arg("-title")
            .arg(title)
            .arg("-message")
            .arg(message)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(NotifyError::CommandFailed {
                code: output.status.code(),
                stderr,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum NotifyError {
    #[error("terminal-notifier binary was not found in PATH")]
    ExecutableNotFound,
    #[error("failed to invoke terminal-notifier: {0}")]
    Io(#[from] std::io::Error),
    #[error("terminal-notifier exited with code {code:?}: {stderr}")]
    CommandFailed { code: Option<i32>, stderr: String },
}
