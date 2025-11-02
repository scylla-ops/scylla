use crate::application::dto::{
    RemoveUserFromOrganizationRequestDto, RemoveUserFromOrganizationResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserOrganizationRepository;
use std::sync::Arc;

pub struct RemoveUserFromOrganizationUseCase<R>
where
    R: UserOrganizationRepository + ?Sized,
{
    user_org_repo: Arc<R>,
}

impl<R> RemoveUserFromOrganizationUseCase<R>
where
    R: UserOrganizationRepository + ?Sized,
{
    pub fn new(user_org_repo: Arc<R>) -> Self {
        Self { user_org_repo }
    }

    pub async fn execute(
        &self,
        request: RemoveUserFromOrganizationRequestDto,
    ) -> DomainResult<RemoveUserFromOrganizationResponseDto> {
        let _ = self
            .user_org_repo
            .find_by_user_and_organization(&request.user_id, &request.organization_id)
            .await?;

        self.user_org_repo
            .remove_user_from_organization(&request.user_id, &request.organization_id)
            .await?;

        Ok(RemoveUserFromOrganizationResponseDto {})
    }
}
