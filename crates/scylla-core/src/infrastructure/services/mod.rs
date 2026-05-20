#[cfg(feature = "hash")]
pub mod argon2_hash_service;
#[cfg(feature = "permission")]
pub mod cedar_permission_service;
#[cfg(feature = "mail")]
pub mod lettre_mailer;
#[cfg(feature = "oauth-github")]
pub mod github_oauth_provider;

#[cfg(feature = "hash")]
pub use argon2_hash_service::Argon2HashService;
#[cfg(feature = "permission")]
pub use cedar_permission_service::CedarPermissionService;
#[cfg(feature = "mail")]
pub use lettre_mailer::LettreMailer;
#[cfg(feature = "oauth-github")]
pub use github_oauth_provider::GitHubOAuthProvider;
