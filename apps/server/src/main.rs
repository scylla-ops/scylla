mod agents_manager;
mod client_connection;
mod constants;
mod server;
mod server_config;

use crate::server_config::ServerConfig;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use protocol::{ApiMessage, Message, Pipeline};
use protocol::serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize)]
struct CommandRequest {
    #[serde(flatten)]
    pipeline: Pipeline,
}

// temporary function to execute a command
async fn execute_command(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CommandRequest>,
) -> String {
    state
        .server_tx
        .send(Message::Api(ApiMessage::ExecutePipeline {
            pipeline: payload.pipeline,
        }))
        .await
        .unwrap();

    "Ok".to_string()
}

struct AppState {
    server_tx: mpsc::Sender<Message>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace")),
        )
        .init();

    info!("Backend server starting");

    let tcp_server_config = ServerConfig::builder()
        .host("127.0.0.1")
        .port(8080)
        .build()?;
    let (mut server, server_tx) = server::Server::with_config(tcp_server_config)
        .await
        .unwrap();

    let tcp_server = tokio::spawn(async move {
        server.run().await.unwrap();
    });

    let app_state = AppState { server_tx };

    let http_addr = "127.0.0.1:3000";
    let http_listener = TcpListener::bind(http_addr).await?;

    let http_server = tokio::spawn(async move {
        info!("HTTP server running on {}", http_addr);

        let command_router = Router::new().route(
            "/execute",
            post(execute_command).with_state(Arc::from(app_state)),
        );

        let app = Router::new()
            .route("/", get(|| async { "Hello, Axum!" }))
            .layer(TraceLayer::new_for_http())
            .merge(command_router);

        axum::serve(http_listener, app).await.unwrap();
    });

    tokio::try_join!(http_server, tcp_server)?;

    Ok(())
}
