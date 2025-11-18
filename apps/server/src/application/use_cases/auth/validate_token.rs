use crate::application::dto::{ValidateTokenRequestDto, ValidateTokenResponseDto};
use crate::application::ports::AuthService;
use crate::domain::errors::DomainResult;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ValidateTokenUseCase<A>
where
    A: AuthService + ?Sized,
{
    auth_service: Arc<A>,
}

impl<A> ValidateTokenUseCase<A>
where
    A: AuthService + ?Sized,
{
    pub async fn execute(
        &self,
        request: ValidateTokenRequestDto,
    ) -> DomainResult<ValidateTokenResponseDto> {
        let is_valid = self.auth_service.validate_token(&request.token).await?;
        Ok(ValidateTokenResponseDto { is_valid })
    }
}
