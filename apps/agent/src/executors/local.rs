use crate::model::executor::{ExecOutput, ExecRequest, Executor, LogEvent, LogStream};
use crate::model::status::{EventKind, PipelineEvent, StepEvent};
use anyhow::Result;
use async_trait::async_trait;
use derive_more::Constructor;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;

#[derive(Constructor, Clone)]
pub struct LocalExecutor;

#[async_trait]
impl Executor for LocalExecutor {
    async fn run_step(&self, req: ExecRequest<'_>) -> Result<ExecOutput> {
        let step = req.step;

        req.status_sink
            .on_event(PipelineEvent::Step(StepEvent {
                id: step.id,
                kind: EventKind::Running,
            }))
            .await;

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

        // stdout
        if let Some(sink) = &req.log_sink {
            let sink_out = sink.clone();
            let mut out_task = None;
            if let Some(mut out_reader) = child.stdout.take().map(BufReader::new) {
                out_task = Some(tokio::spawn(async move {
                    let mut buf = String::new();
                    loop {
                        buf.clear();
                        let n = out_reader.read_line(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        sink_out
                            .on_log_chunk(LogEvent {
                                stream: LogStream::Stdout,
                                chunk: buf.clone(),
                            })
                            .await;
                    }
                    Ok::<(), anyhow::Error>(())
                }));
            }
        }

        let status = child.wait().await?;

        req.status_sink
            .on_event(PipelineEvent::Step(StepEvent {
                id: step.id,
                kind: if status.success() {
                    EventKind::Succeeded
                } else {
                    EventKind::Failed
                },
            }))
            .await;

        Ok(ExecOutput { status })
    }
}
