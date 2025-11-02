use crate::application::ports::AuthService;
use crate::config::AuthConfig;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use base64::Engine;
use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::{Generate, SymmetricKey};
use pasetors::token::UntrustedToken;
use pasetors::{Local, local, version4::V4};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct PasetoAuthService {
    key: SymmetricKey<V4>,
    token_duration: Duration,
    // blacklist stores token hashes for revoked tokens
    blacklist: Arc<RwLock<HashSet<String>>>,
}

impl PasetoAuthService {
    pub fn new(key: SymmetricKey<V4>, token_duration: Duration) -> Self {
        Self {
            key,
            token_duration,
            blacklist: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn with_generated_key() -> Self {
        // key generation should never fail, but whatever
        let key = SymmetricKey::<V4>::generate()
            .unwrap_or_else(|_| panic!("Failed to generate encryption key"));
        Self::new(key, Duration::from_secs(3600 * 24)) // 24 hours
    }

    /// Create from configuration
    pub fn from_config(config: &AuthConfig) -> DomainResult<Self> {
        let token_duration = Duration::from_secs(config.token_duration_seconds);

        let key = if let Some(key_b64) = &config.token_key {
            // Decode base64 string to bytes and create key from bytes
            let key_bytes = base64::engine::general_purpose::STANDARD
                .decode(key_b64.as_str())
                .map_err(|e| {
                    DomainError::internal(format!("Failed to decode base64 key from config: {}", e))
                })?;
            SymmetricKey::<V4>::from(&key_bytes).map_err(|e| {
                DomainError::internal(format!("Failed to create symmetric key from config: {}", e))
            })?
        } else {
            // generate new key if not provided
            SymmetricKey::<V4>::generate().map_err(|e| {
                DomainError::internal(format!("Failed to generate encryption key: {}", e))
            })?
        };

        Ok(Self::new(key, token_duration))
    }

    // hash token for blacklist storage (using sha2 for consistent hashing)
    fn hash_token(&self, token: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        // convert bytes to hex string
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[async_trait]
impl AuthService for PasetoAuthService {
    async fn generate_token(&self, user_id: &UserId) -> DomainResult<String> {
        let mut claims = Claims::new()
            .map_err(|e| DomainError::internal(format!("Failed to create claims: {}", e)))?;

        claims
            .add_additional("user_id", user_id.as_str())
            .map_err(|e| DomainError::internal(format!("Failed to add user_id claim: {}", e)))?;

        let expiration_duration = chrono::Duration::from_std(self.token_duration)
            .map_err(|e| DomainError::internal(format!("Invalid token duration: {}", e)))?;
        claims
            .expiration(&(chrono::Utc::now() + expiration_duration).to_rfc3339())
            .map_err(|e| DomainError::internal(format!("Failed to set expiration: {}", e)))?;

        let token = local::encrypt(&self.key, &claims, None, None)
            .map_err(|e| DomainError::internal(format!("Failed to encrypt token: {}", e)))?;

        Ok(token)
    }

    async fn validate_token(&self, token: &str) -> DomainResult<bool> {
        // check if token is in blacklist
        let token_hash = self.hash_token(token);
        let blacklist = self.blacklist.read().await;
        if blacklist.contains(&token_hash) {
            return Err(DomainError::unauthorized("Token has been revoked"));
        }
        drop(blacklist); // release lock early

        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| DomainError::unauthorized(format!("Invalid token format: {}", e)))?;

        let trusted_token =
            local::decrypt(&self.key, &untrusted_token, &validation_rules, None, None).map_err(
                |e| DomainError::unauthorized(format!("Token validation failed: {}", e)),
            )?;

        let claims = trusted_token
            .payload_claims()
            .ok_or_else(|| DomainError::unauthorized("Token has no claims"))?;

        let user_id_str = claims
            .get_claim("user_id")
            .ok_or_else(|| DomainError::unauthorized("Token missing user_id claim"))?
            .as_str()
            .ok_or_else(|| DomainError::unauthorized("user_id claim is not a string"))?;

        Ok(!user_id_str.is_empty())
    }

    async fn extract_user_id(&self, token: &str) -> DomainResult<UserId> {
        // check if token is in blacklist
        let token_hash = self.hash_token(token);
        let blacklist = self.blacklist.read().await;
        if blacklist.contains(&token_hash) {
            return Err(DomainError::unauthorized("Token has been revoked"));
        }
        drop(blacklist); // release lock early

        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| DomainError::unauthorized(format!("Invalid token format: {}", e)))?;

        let trusted_token =
            local::decrypt(&self.key, &untrusted_token, &validation_rules, None, None).map_err(
                |e| DomainError::unauthorized(format!("Token validation failed: {}", e)),
            )?;

        let claims = trusted_token
            .payload_claims()
            .ok_or_else(|| DomainError::unauthorized("Token has no claims"))?;

        let user_id_str = claims
            .get_claim("user_id")
            .ok_or_else(|| DomainError::unauthorized("Token missing user_id claim"))?
            .as_str()
            .ok_or_else(|| DomainError::unauthorized("user_id claim is not a string"))?;

        Ok(UserId::new(user_id_str.to_string()))
    }

    async fn is_token_expired(&self, token: &str) -> DomainResult<bool> {
        match self.validate_token(token).await {
            Ok(_) => Ok(false),
            Err(DomainError::Unauthorized(_)) => Ok(true),
            Err(e) => Err(e),
        }
    }

    async fn revoke_token(&self, token: &str) -> DomainResult<()> {
        // validate token first to ensure it's valid before revoking
        // this prevents invalid tokens from being added to the blacklist
        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| DomainError::unauthorized(format!("Invalid token format: {}", e)))?;

        local::decrypt(&self.key, &untrusted_token, &validation_rules, None, None)
            .map_err(|e| DomainError::unauthorized(format!("Token validation failed: {}", e)))?;

        // add token hash to blacklist
        let token_hash = self.hash_token(token);
        let mut blacklist = self.blacklist.write().await;
        blacklist.insert(token_hash);

        Ok(())
    }
}
