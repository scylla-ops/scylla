mod agents_manager;
mod constants;
mod core;
mod server_config;

use crate::agents_manager::AgentsManager;
use crate::constants::MAX_CHANNEL_SIZE;
use crate::server_config::CoreConfig;
use axum::extract::ws::{Message as WsMessage, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, routing::get};
use clap::Parser;
use futures::{SinkExt, StreamExt};
use protocol::serde::Deserialize;
use protocol::serde_json;
use protocol::uuid::Uuid;
use protocol::{
    AgentMessage, ApiMessage, Message, Pipeline,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use anyhow::{Context, Result, anyhow};

#[derive(Deserialize)]
struct CommandRequest {
    #[serde(flatten)]
    pipeline: Pipeline,
}

/// Scylla Server - The core server for the Scylla CI/CD system
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Print an example configuration file and exit
    #[arg(short = 'e', long = "print-example-config")]
    print_example_config: bool,

    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<String>,
}

// Function to execute a command
async fn execute_command(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CommandRequest>,
) -> String {
    if let Err(e) = state
        .core_tx
        .send(Message::Api(ApiMessage::ExecutePipeline {
            pipeline: payload.pipeline,
        }))
        .await
    {
        error!("Failed to send pipeline execution message: {}", e);
        return format!("Error: {}", e);
    }

    "Ok".to_string()
}

// Handle WebSocket connections from agents
async fn handle_agent_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_agent_socket(socket, state))
}

// Process WebSocket connection from an agent
async fn handle_agent_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Create a new UUID for this agent
    let agent_id = Uuid::new_v4();

    // Create channels for communication
    let (client_tx, mut client_rx) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);

    // Add agent to the manager
    state
        .agents_manager
        .lock()
        .await
        .add_agent(agent_id, client_tx.clone());

    // Send registration message to the agent
    let register_msg = Message::Agent(AgentMessage::Register { agent_id });
    if let Ok(json) = serde_json::to_string(&register_msg) {
        if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
            error!("Failed to send registration message: {}", e);
            state.agents_manager.lock().await.remove_agent(agent_id);
            return;
        }
    }

    // Task to forward messages from server to agent
    let mut send_task = tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&message) {
                if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                    error!("Error sending message to agent: {}", e);
                    break;
                }
            }
        }
    });

    // Task to process messages from agent
    let core_tx = state.core_tx.clone();
    let agents_manager = state.agents_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(WsMessage::Text(text)) => match serde_json::from_str::<Message>(&text) {
                    Ok(message) => {
                        if let Err(e) = core_tx.send(message).await {
                            error!("Failed to forward message from agent: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse message from agent: {}", e);
                    }
                },
                Ok(WsMessage::Close(_)) => {
                    debug!("WebSocket closed by agent");
                    break;
                }
                _ => {}
            }
        }

        // Remove agent when connection is closed
        agents_manager.lock().await.remove_agent(agent_id);
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }
}

// Process messages using the Core
async fn handle_messages(
    server_rx: mpsc::Receiver<Message>,
    agents_manager: Arc<Mutex<AgentsManager>>,
    jobs: Arc<Mutex<HashMap<Uuid, protocol::Job>>>,
    core_config: CoreConfig,
) -> Result<()> {
    let (core_tx, _) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);

    let mut core = core::Core::new(core_config, server_rx, agents_manager, jobs);

    core.run()
        .await
        .with_context(|| "Failed to run core")?;

    Ok(())
}

struct AppState {
    core_tx: mpsc::Sender<Message>,
    agents_manager: Arc<Mutex<AgentsManager>>,
    jobs: Arc<Mutex<HashMap<Uuid, protocol::Job>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .init();

    // Parse command-line arguments using clap
    let args = Args::parse();

    // Handle the print-example-config flag
    if args.print_example_config {
        CoreConfig::print_example_toml();
        return Ok(());
    }

    info!("Core starting");

    // Create channels for message passing
    let (core_tx, core_rx) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);

    // Initialize shared state
    let agents_manager = Arc::new(Mutex::new(AgentsManager::new()));
    let jobs = Arc::new(Mutex::new(HashMap::new()));

    // Load configuration from file if specified, otherwise use default
    let core_config = match args.config {
        Some(config_path) => {
            match CoreConfig::from_toml_file(&config_path) {
                Ok(config) => {
                    info!("Loaded configuration from {}", config_path);
                    config
                },
                Err(e) => {
                    error!("Failed to load configuration from {}: {}", config_path, e);
                    info!("Falling back to default configuration");
                    CoreConfig::builder().host("127.0.0.1").port(3000).build()
                        .map_err(|e| anyhow!("{}", e))?
                }
            }
        },
        None => CoreConfig::builder().host("127.0.0.1").port(3000).build()
            .map_err(|e| anyhow!("{}", e))?,
    };

    // Create app state for HTTP and WebSocket handlers
    let app_state = Arc::new(AppState {
        core_tx: core_tx.clone(),
        agents_manager: agents_manager.clone(),
        jobs: jobs.clone(),
    });

    // Start message handler task
    let core_config_clone = core_config.clone();
    let message_handler = tokio::spawn(async move {
        if let Err(e) = handle_messages(
            core_rx,
            agents_manager.clone(),
            jobs.clone(),
            core_config_clone,
        )
        .await
        {
            error!("Message handler error: {}", e);
        }
    });

    let listener = TcpListener::bind(core_config.addr).await?;
    info!("Core running on {}", core_config.addr);

    // Create router with HTTP and WebSocket routes
    let app = Router::new()
        .route("/", get(|| async { "Hello, Scylla Core!" }))
        .route("/ws/agent", get(handle_agent_websocket))
        .route("/execute", post(execute_command))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // Run the server and wait for it to complete
    let server_task = axum::serve(listener, app);

    tokio::select! {
        result = server_task => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = message_handler => {
            info!("Message handler task completed");
        }
    }

    Ok(())
}
