use crate::domain::ids::{AppId, TriggerId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobOriginKind {
    Human,
    App,
    Cron,
    Webhook,
}

impl JobOriginKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::App => "app",
            Self::Cron => "cron",
            Self::Webhook => "webhook",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobOrigin {
    Human {
        user_id: UserId,
    },
    App {
        app_id: AppId,
    },
    Cron {
        trigger_id: TriggerId,
    },
    Webhook {
        trigger_id: TriggerId,
        delivery_id: Option<String>,
    },
}

impl JobOrigin {
    #[must_use]
    pub fn kind(&self) -> JobOriginKind {
        match self {
            Self::Human { .. } => JobOriginKind::Human,
            Self::App { .. } => JobOriginKind::App,
            Self::Cron { .. } => JobOriginKind::Cron,
            Self::Webhook { .. } => JobOriginKind::Webhook,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_round_trips_with_kind_tag() {
        let origin = JobOrigin::Human {
            user_id: UserId::new("u1"),
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert!(json.contains(r#""kind":"human""#), "{json}");
        assert!(json.contains(r#""user_id":"u1""#), "{json}");

        let back: JobOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(back, origin);
        assert_eq!(back.kind(), JobOriginKind::Human);
    }

    #[test]
    fn app_round_trips_with_kind_tag() {
        let origin = JobOrigin::App {
            app_id: AppId::new("a1"),
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert!(json.contains(r#""kind":"app""#), "{json}");
        assert!(json.contains(r#""app_id":"a1""#), "{json}");

        let back: JobOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(back, origin);
        assert_eq!(back.kind(), JobOriginKind::App);
    }

    #[test]
    fn cron_round_trips_with_kind_tag() {
        let origin = JobOrigin::Cron {
            trigger_id: TriggerId::new("t1"),
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert!(json.contains(r#""kind":"cron""#), "{json}");
        assert!(json.contains(r#""trigger_id":"t1""#), "{json}");

        let back: JobOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(back, origin);
        assert_eq!(back.kind(), JobOriginKind::Cron);
    }

    #[test]
    fn webhook_round_trips_with_delivery_id() {
        let origin = JobOrigin::Webhook {
            trigger_id: TriggerId::new("t1"),
            delivery_id: Some("d-42".to_string()),
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert!(json.contains(r#""kind":"webhook""#), "{json}");

        let back: JobOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(back, origin);
        assert_eq!(back.kind(), JobOriginKind::Webhook);
    }

    #[test]
    fn webhook_round_trips_without_delivery_id() {
        let origin = JobOrigin::Webhook {
            trigger_id: TriggerId::new("t1"),
            delivery_id: None,
        };
        let back: JobOrigin =
            serde_json::from_str(&serde_json::to_string(&origin).unwrap()).unwrap();
        assert_eq!(back, origin);
    }
}
