use crate::application::dto::{DeleteOrganizationRequestDto, DeleteOrganizationResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct DeleteOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    org_repo: Arc<R>,
}

impl<R> DeleteOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: DeleteOrganizationRequestDto,
    ) -> DomainResult<DeleteOrganizationResponseDto> {
        let _ = self.org_repo.find_by_id(&request.organization_id).await?;

        self.org_repo.delete(&request.organization_id).await?;
        Ok(DeleteOrganizationResponseDto {})
    }
}
