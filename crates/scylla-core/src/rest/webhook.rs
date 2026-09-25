use crate::application::{IngestOutcome, IngestWebhook, WebhookIngressUseCases};
use crate::domain::errors::DomainError;
use crate::domain::ids::TriggerId;
use crate::rest::adapter::run_public;
use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use scylla_extension::Actions;
use std::collections::HashMap;
use std::sync::Arc;

const DELIVERY_HEADERS: [&str; 2] = ["X-Scylla-Delivery", "X-GitHub-Delivery"];
const EVENT_HEADERS: [&str; 2] = ["X-Scylla-Event", "X-GitHub-Event"];

#[derive(Clone)]
struct Webhooks {
    actions: Arc<Actions>,
    ingress: Arc<WebhookIngressUseCases>,
}

pub fn router(actions: Arc<Actions>, ingress: Arc<WebhookIngressUseCases>) -> Router {
    Router::new()
        .route("/webhooks/{trigger_id}", post(handle))
        .with_state(Webhooks { actions, ingress })
}

async fn handle(
    State(webhooks): State<Webhooks>,
    Path(trigger_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, &'static str) {
    let command = ingest_webhook(&trigger_id, &headers, body);
    match run_public(&webhooks.actions, &*webhooks.ingress, command).await {
        // A missing agent is not an error: the run is minted and pending.
        Ok(IngestOutcome::Fired(_)) => (StatusCode::ACCEPTED, "accepted"),
        Ok(IngestOutcome::Duplicate) => (StatusCode::OK, "duplicate"),
        Ok(IngestOutcome::Ping) => (StatusCode::OK, "pong"),
        Err(DomainError::NotFound { .. }) => (StatusCode::NOT_FOUND, "not found"),
        Err(DomainError::Unauthorized(_)) => (StatusCode::UNAUTHORIZED, "invalid signature"),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
    }
}

/// Each header keeps its first value, if that value is text, under its lowercase name.
fn ingest_webhook(trigger_id: &str, headers: &HeaderMap, body: Bytes) -> IngestWebhook {
    let first_header = |names: &[&str]| {
        names.iter().find_map(|h| {
            headers
                .get(*h)
                .and_then(|v| v.to_str().ok())
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
        })
    };
    let values: HashMap<_, _> = headers
        .keys()
        .filter_map(|name| {
            let value = headers.get(name)?.to_str().ok()?;
            Some((name.as_str().to_owned(), value.to_owned()))
        })
        .collect();
    IngestWebhook {
        trigger_id: TriggerId::new(trigger_id),
        headers: values,
        delivery_id: first_header(&DELIVERY_HEADERS),
        event: first_header(&EVENT_HEADERS),
        body: body.into(),
    }
}
