use crate::application::oauth::{OAuthProvider, OAuthUserInfo};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::Email;
use async_trait::async_trait;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

const AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";
const EMAILS_URL: &str = "https://api.github.com/user/emails";

/// Fully-configured oauth2 client type (all endpoints set, no
/// device/introspection/revocation), kept as an alias to avoid repeating the
/// long typestate signature.
type ConfiguredClient = oauth2::Client<
    oauth2::basic::BasicErrorResponse,
    oauth2::basic::BasicTokenResponse,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

/// GitHub OAuth provider: `oauth2` drives the authorize/token protocol, while
/// `reqwest` performs the token exchange transport and the user-info fetch.
pub struct GitHubOAuthProvider {
    client: ConfiguredClient,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: u64,
    login: String,
    email: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

impl GitHubOAuthProvider {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> DomainResult<Self> {
        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(
                AuthUrl::new(AUTHORIZE_URL.to_string())
                    .map_err(|e| DomainError::infrastructure(format!("auth url: {e}")))?,
            )
            .set_token_uri(
                TokenUrl::new(TOKEN_URL.to_string())
                    .map_err(|e| DomainError::infrastructure(format!("token url: {e}")))?,
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_uri)
                    .map_err(|e| DomainError::infrastructure(format!("redirect uri: {e}")))?,
            );
        // oauth2 requires the HTTP client to forbid redirects (SSRF safety).
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("scylla")
            .build()
            .map_err(|e| DomainError::infrastructure(format!("http client: {e}")))?;
        Ok(Self { client, http })
    }

    async fn fetch_primary_email(&self, access_token: &str) -> Option<Email> {
        let emails: Vec<GitHubEmail> = self
            .http
            .get(EMAILS_URL)
            .bearer_auth(access_token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;
        emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .and_then(|e| Email::new(e.email).ok())
    }
}

#[async_trait]
impl OAuthProvider for GitHubOAuthProvider {
    fn authorize_url(&self, state: &str) -> DomainResult<String> {
        let state = state.to_string();
        let (url, _csrf) = self
            .client
            .authorize_url(move || CsrfToken::new(state))
            .add_scope(Scope::new("read:user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
            .url();
        Ok(url.to_string())
    }

    async fn exchange_code(&self, code: &str) -> DomainResult<OAuthUserInfo> {
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&self.http)
            .await
            .map_err(|e| DomainError::infrastructure(format!("token exchange: {e}")))?;
        let access = token.access_token().secret();

        let gh_user: GitHubUser = self
            .http
            .get(USER_URL)
            .bearer_auth(access)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| DomainError::infrastructure(format!("github user fetch: {e}")))?
            .error_for_status()
            .map_err(|e| DomainError::infrastructure(format!("github user status: {e}")))?
            .json()
            .await
            .map_err(|e| DomainError::infrastructure(format!("github user decode: {e}")))?;

        let email = match gh_user.email.and_then(|e| Email::new(e).ok()) {
            Some(email) => Some(email),
            None => self.fetch_primary_email(access).await,
        };

        Ok(OAuthUserInfo {
            provider_user_id: gh_user.id.to_string(),
            email,
            login: gh_user.login,
        })
    }
}
