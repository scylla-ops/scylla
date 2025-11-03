use crate::application::dto::{RemoveUserFromProjectRequestDto, RemoveUserFromProjectResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserProjectRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct RemoveUserFromProjectUseCase<R>
where
    R: UserProjectRepository + ?Sized,
{
    user_project_repo: Arc<R>,
}

impl<R> RemoveUserFromProjectUseCase<R>
where
    R: UserProjectRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: RemoveUserFromProjectRequestDto,
    ) -> DomainResult<RemoveUserFromProjectResponseDto> {
        self.user_project_repo
            .remove_user_from_project(&request.user_id, &request.project_id)
            .await?;
        Ok(RemoveUserFromProjectResponseDto {})
    }
}
