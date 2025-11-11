use crate::application::dto::{RevokeTokenRequestDto, RevokeTokenResponseDto};
use crate::application::ports::AuthService;
use crate::domain::errors::DomainResult;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct RevokeTokenUseCase<A>
where
    A: AuthService + ?Sized,
{
    auth_service: Arc<A>,
}

impl<A> RevokeTokenUseCase<A>
where
    A: AuthService + ?Sized,
{
    pub async fn execute(
        &self,
        request: RevokeTokenRequestDto,
    ) -> DomainResult<RevokeTokenResponseDto> {
        self.auth_service.revoke_token(&request.token).await?;
        Ok(RevokeTokenResponseDto {})
    }
}
