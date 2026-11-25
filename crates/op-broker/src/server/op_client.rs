use std::{path::PathBuf, time::Duration};
use thiserror::Error;
use tokio::{process::Command, time::timeout};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct OpClient {
    binary: PathBuf,
    timeout: Duration,
}

impl OpClient {
    pub fn discover() -> Result<Self, OpError> {
        let binary = which::which("op").map_err(|_| OpError::ExecutableNotFound)?;
        Ok(Self {
            binary,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    pub async fn read(&self, reference: &str) -> Result<String, OpError> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("read").arg(reference);
        let output = timeout(self.timeout, cmd.output())
            .await
            .map_err(|_| OpError::Timeout)??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(OpError::CommandFailed {
                code: output.status.code(),
                stderr,
            });
        }

        let stdout = String::from_utf8(output.stdout).map_err(|_| OpError::InvalidUtf8)?;
        let trimmed = stdout.trim_end_matches(['\n', '\r']).to_owned();
        if trimmed.is_empty() {
            return Err(OpError::EmptyResponse);
        }
        Ok(trimmed)
    }
}

#[derive(Debug, Error)]
pub enum OpError {
    #[error("op CLI binary was not found in PATH")]
    ExecutableNotFound,
    #[error("op CLI invocation timed out")]
    Timeout,
    #[error("failed to invoke op CLI: {0}")]
    Io(#[from] std::io::Error),
    #[error("op CLI exited with code {code:?}: {stderr}")]
    CommandFailed { code: Option<i32>, stderr: String },
    #[error("op CLI output was not valid UTF-8")]
    InvalidUtf8,
    #[error("op CLI returned an empty response")]
    EmptyResponse,
}
