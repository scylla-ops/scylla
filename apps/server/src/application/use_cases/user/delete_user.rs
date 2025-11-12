use crate::application::dto::{DeleteUserRequestDto, DeleteUserResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct DeleteUserUseCase<R>
where
    R: UserRepository + ?Sized,
{
    user_repo: Arc<R>,
}

impl<R> DeleteUserUseCase<R>
where
    R: UserRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: DeleteUserRequestDto,
    ) -> DomainResult<DeleteUserResponseDto> {
        let _ = self.user_repo.find_by_id(&request.user_id).await?;

        self.user_repo.delete(&request.user_id).await?;
        Ok(DeleteUserResponseDto {})
    }
}
