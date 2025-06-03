use crate::error::{self, Result};
use async_trait::async_trait;
use tokio::process::Command as TokioCommand;

#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, cmd: &str) -> Result<String>;
}

#[derive(Debug, Clone, Default)]
pub struct ShellCommandExecutor;

#[async_trait]
impl CommandExecutor for ShellCommandExecutor {
    async fn execute(&self, cmd: &str) -> Result<String> {
        let output = TokioCommand::new("sh").arg("-c").arg(cmd).output().await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(error::command_error(String::from_utf8_lossy(
                &output.stderr,
            )))
        }
    }
}
