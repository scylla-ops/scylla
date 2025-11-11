use crate::executors::local::LocalExecutor;
use crate::model::executor::{LogEvent, LogSink, PipelineRunnerBuilder};
use crate::model::status::{PipelineEvent, StatusSink};
use anyhow::Context;
use derive_more::Constructor;
use futures_util::StreamExt;
use protocol::services::orchestrator::orchestrator_client::OrchestratorClient;
use protocol::services::orchestrator::{Job, WorkerId};
use protocol::toml;
use protocol::tonic::Request;
use protocol::tonic::transport::Channel;
use protocol::uuid::Uuid;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info};

#[derive(Clone, Debug, Constructor)]
pub struct ChannelLogSink {
    tx: mpsc::Sender<LogEvent>,
}
#[async_trait::async_trait]
impl LogSink for ChannelLogSink {
    async fn on_log_chunk(&self, ev: LogEvent) {
        let _ = self.tx.send(ev).await;
    }
}

#[derive(Clone, Debug, Constructor)]
pub struct ChannelStatusSink {
    tx: mpsc::Sender<PipelineEvent>,
}

#[async_trait::async_trait]
impl StatusSink for ChannelStatusSink {
    async fn on_event(&self, ev: PipelineEvent) {
        let _ = self.tx.send(ev).await;
    }
}

pub struct Agent {
    endpoint: String,
    token: String,
}

impl Agent {
    pub fn new(grpc_addr: SocketAddr) -> Self {
        // Token configurable via ORCH_TOKEN, fallback sur valeur par défaut
        let token = std::env::var("ORCH_TOKEN").unwrap_or_else(|_| "not a good token".to_string());
        Self {
            endpoint: format!("http://{}", grpc_addr),
            token,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let worker_id = WorkerId {
            id: Uuid::new_v4().to_string(),
        };

        loop {
            match self.get_and_handle_single_job(&worker_id).await {
                Ok(()) => {
                    info!("Job processed, waiting for next job...");
                }
                Err(e) => {
                    error!("Error while processing job: {e:#}");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn get_and_handle_single_job(
        &mut self,
        worker_id: &WorkerId,
    ) -> Result<(), Box<dyn Error>> {
        let channel: Channel = Channel::from_shared(self.endpoint.clone())?
            .connect()
            .await?;

        let mut jobs_client = OrchestratorClient::with_interceptor(channel.clone(), {
            let token = self.token.clone();
            move |mut req: Request<()>| {
                req.metadata_mut()
                    .insert("x-orch-token", token.parse().unwrap());
                Ok(req)
            }
        });

        let mut stream = jobs_client
            .subscribe_job(Request::new(worker_id.clone()))
            .await?
            .into_inner();

        if let Some(job) = stream.message().await? {
            self.handle_job(channel, job).await?;
        }

        Ok(())
    }

    async fn handle_job(&mut self, channel: Channel, job: Job) -> Result<(), Box<dyn Error>> {
        let job_entry: protocol::job::JobEntry =
            toml::from_str(&job.job_toml).context("Failed to parse job TOML")?;

        let mut status_client = OrchestratorClient::with_interceptor(channel, {
            let token = self.token.clone();
            move |mut req: Request<()>| {
                req.metadata_mut()
                    .insert("x-orch-token", token.parse().unwrap());
                Ok(req)
            }
        });

        let (job_status_tx, job_status_rx) = mpsc::channel::<PipelineEvent>(64);

        tokio::spawn(async move {
            let outbound = ReceiverStream::new(job_status_rx).filter_map(|ev| async move {
                let converted = TryInto::try_into(ev);
                match converted {
                    Ok(ok) => Some(ok),
                    Err(err) => {
                        tracing::warn!("Failed to convert PipelineEvent: {err}");
                        None
                    }
                }
            });

            status_client
                .report_status(Request::new(outbound))
                .await
                .map_err(|e| anyhow::anyhow!("ReportStatus failed: {e}"))
        });

        let status_sink = ChannelStatusSink::new(job_status_tx);

        let (log_event_tx, log_event_rx) = mpsc::channel::<LogEvent>(64);
        let log_sink = ChannelLogSink::new(log_event_tx);

        tokio::spawn(async move {
            // temporary
            let mut logs = ReceiverStream::new(log_event_rx);
            while let Some(log) = logs.next().await {
                debug!("[LOG] {:?}", log);
            }
        });

        let executor = LocalExecutor::new();

        let mut builder = PipelineRunnerBuilder::default();
        builder
            .executor(executor)
            .status_sink(Arc::new(status_sink))
            .log_sink(Some(Arc::new(log_sink)))
            .default_workdir(Some(".".into()))
            .default_env(HashMap::default())
            .job(job_entry);

        let mut runner = builder.build().unwrap();

        let _res = runner.run_job().await;

        Ok(())
    }
}
