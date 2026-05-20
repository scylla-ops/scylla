#[macro_use]
pub mod macros;

pub mod agent_handler;
pub mod auth_handler;
pub mod config_handler;
pub mod grant_handler;
#[cfg(feature = "invitations")]
pub mod invitation_handler;
pub mod job_handler;
#[cfg(feature = "oauth-github")]
pub mod oauth_handler;
pub mod organization_handler;
pub mod pipeline_handler;
pub mod policy_handler;
pub mod project_handler;
#[cfg(feature = "signup")]
pub mod registration_handler;
pub mod role_handler;
pub mod user_handler;

pub use agent_handler::AgentHandler;
pub use auth_handler::AuthHandler;
pub use config_handler::ConfigHandler;
pub use grant_handler::GrantHandler;
#[cfg(feature = "invitations")]
pub use invitation_handler::InvitationHandler;
pub use job_handler::JobHandler;
#[cfg(feature = "oauth-github")]
pub use oauth_handler::OAuthHandler;
pub use organization_handler::OrganizationHandler;
pub use pipeline_handler::PipelineHandler;
pub use policy_handler::PolicyHandler;
pub use project_handler::ProjectHandler;
#[cfg(feature = "signup")]
pub use registration_handler::RegistrationHandler;
pub use role_handler::RoleHandler;
pub use user_handler::UserHandler;
