use super::{CronSpec, WebhookSpec};
use serde::{Deserialize, Serialize};

/// The discriminant of a [`TriggerSource`], denormalized into its own column for
/// the engine's due-scan and routing (mirrors `grants.principal_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind {
    Cron,
    Webhook,
}

impl TriggerKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cron => "cron",
            Self::Webhook => "webhook",
        }
    }
}

/// What initiates a trigger's run. A sealed, extensible set — `Poll` joins in
/// v0.4 as a new arm. Internally tagged with `kind` so the persisted JSONB is
/// self-describing: `{"kind":"cron","expression":"..."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerSource {
    Cron(CronSpec),
    Webhook(WebhookSpec),
}

impl TriggerSource {
    #[must_use]
    pub fn kind(&self) -> TriggerKind {
        match self {
            Self::Cron(_) => TriggerKind::Cron,
            Self::Webhook(_) => TriggerKind::Webhook,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_source_round_trips_with_kind_tag() {
        let source = TriggerSource::Cron(CronSpec::new("0 9 * * *").unwrap());
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""kind":"cron""#), "{json}");
        assert!(json.contains(r#""expression":"0 9 * * *""#), "{json}");

        let back: TriggerSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, source);
        assert_eq!(back.kind(), TriggerKind::Cron);
    }

    #[test]
    fn webhook_source_round_trips_with_kind_tag() {
        let source = TriggerSource::Webhook(WebhookSpec::new(Some("X-Hub-Signature-256".into())).unwrap());
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""kind":"webhook""#), "{json}");

        let back: TriggerSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, source);
        assert_eq!(back.kind(), TriggerKind::Webhook);
    }
}
