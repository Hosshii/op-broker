use std::{path::PathBuf, time::Duration};
use thiserror::Error;
use tokio::{process::Command, time::timeout};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct ExecClient {
    binary: PathBuf,
    timeout: Duration,
}

pub struct ExecOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ExecClient {
    pub fn from_path(path: PathBuf) -> Result<Self, ExecError> {
        if !path.exists() {
            return Err(ExecError::ExecutableNotFound(path));
        }
        if !path.is_file() {
            return Err(ExecError::NotAFile(path));
        }
        Ok(Self {
            binary: path,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    pub async fn execute(&self, args: &[String]) -> Result<ExecOutput, ExecError> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(args);
        let output = timeout(self.timeout, cmd.output())
            .await
            .map_err(|_| ExecError::Timeout)??;

        let stdout = String::from_utf8(output.stdout).map_err(|_| ExecError::InvalidUtf8)?;
        let stderr = String::from_utf8(output.stderr).map_err(|_| ExecError::InvalidUtf8)?;
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(ExecOutput {
            exit_code,
            stdout: stdout.trim_end_matches('\n').to_owned(),
            stderr: stderr.trim_end_matches('\n').to_owned(),
        })
    }
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("executable not found: {0}")]
    ExecutableNotFound(PathBuf),
    #[error("path is not a file: {0}")]
    NotAFile(PathBuf),
    #[error("command execution timed out")]
    Timeout,
    #[error("failed to execute command: {0}")]
    Io(#[from] std::io::Error),
    #[error("command output was not valid UTF-8")]
    InvalidUtf8,
}
