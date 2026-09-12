//! Driven adapters that need no database: the in-process messaging (agent
//! registry, job-log fan-out) and the service adapters (password hashing,
//! secret encryption, cron, OAuth, SMTP). The Postgres adapters live in
//! `scylla-db`, the Cedar adapter in `scylla-auth`.

pub mod messaging;
pub mod services;

pub use messaging::{InMemoryAgentRegistry, InMemoryJobLogStream};

pub use services::{ChaChaSecretCipher, CronScheduleService, GitHubOAuthProvider, LettreMailer};

pub use services::Argon2HashService;

pub use services::CedarPermissionService;
