use scylla_protocol::services::config::{
    GetServerConfigRequest, ServerConfigResponse, config_service_server::ConfigService,
};
use tonic::{Request, Response, Status};

/// Reports which optional, cargo-gated features this server was built with.
/// Public + stateless: every value is a compile-time `cfg!` flag, so the
/// handler holds no dependencies. The frontend reads this at boot to show/hide
/// UI (register, GitHub login, …) for the PaaS vs SaaS edition.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigHandler;

#[async_trait::async_trait]
impl ConfigService for ConfigHandler {
    async fn get_server_config(
        &self,
        _request: Request<GetServerConfigRequest>,
    ) -> Result<Response<ServerConfigResponse>, Status> {
        Ok(Response::new(ServerConfigResponse {
            signup_enabled: cfg!(feature = "signup"),
            invitations_enabled: cfg!(feature = "invitations"),
            oauth_github_enabled: cfg!(feature = "oauth-github"),
            metering_enabled: cfg!(feature = "metering"),
            mail_enabled: cfg!(feature = "mail"),
            agent_org_scope_enabled: cfg!(feature = "agent-org-scope"),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reports_each_field_for_its_own_feature() {
        let resp = ConfigHandler
            .get_server_config(Request::new(GetServerConfigRequest {}))
            .await
            .unwrap()
            .into_inner();
        // Guards against cross-wiring a flag to the wrong cargo feature.
        assert_eq!(resp.signup_enabled, cfg!(feature = "signup"));
        assert_eq!(resp.invitations_enabled, cfg!(feature = "invitations"));
        assert_eq!(resp.oauth_github_enabled, cfg!(feature = "oauth-github"));
        assert_eq!(resp.metering_enabled, cfg!(feature = "metering"));
        assert_eq!(resp.mail_enabled, cfg!(feature = "mail"));
        assert_eq!(resp.agent_org_scope_enabled, cfg!(feature = "agent-org-scope"));
    }
}
