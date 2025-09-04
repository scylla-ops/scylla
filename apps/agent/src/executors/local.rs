use crate::model::executor::{ExecOutput, ExecRequest, Executor};
use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[derive(Constructor)]
pub struct LocalExecutor;

#[async_trait]
impl Executor for LocalExecutor {
    async fn run_step(&self, req: ExecRequest<'_>) -> Result<ExecOutput> {
        let step = req.step;

        let mut cmd: Command = step.shell.clone().into();

        let full_command = std::iter::once(&step.command)
            .chain(step.args.iter())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        cmd.arg(full_command);

        if let Some(dir) = req.workdir {
            cmd.current_dir(dir);
        }

        if let Some(vars) = req.env {
            for (k, v) in vars {
                cmd.env(k, v);
            }
        }

        cmd.kill_on_drop(true);

        let mut child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let Some(mut out) = child.stdout.take() {
            let mut buf = Vec::new();
            out.read_to_end(&mut buf).await?;
            stdout = String::from_utf8_lossy(&buf).to_string();
        }

        if let Some(mut err) = child.stderr.take() {
            let mut buf = Vec::new();
            err.read_to_end(&mut buf).await?;
            stderr = String::from_utf8_lossy(&buf).to_string();
        }

        let status = child.wait().await?;
        let code = status
            .code()
            .unwrap_or_else(|| if status.success() { 0 } else { 1 });

        Ok(ExecOutput {
            status_code: code,
            stdout,
            stderr,
        })
    }
}
