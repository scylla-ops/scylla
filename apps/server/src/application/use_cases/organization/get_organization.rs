use crate::application::dto::{GetOrganizationRequestDto, OrganizationResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct GetOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    org_repo: Arc<R>,
}

impl<R> GetOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: GetOrganizationRequestDto,
    ) -> DomainResult<OrganizationResponseDto> {
        let organization = self.org_repo.find_by_id(&request.organization_id).await?;

        Ok(OrganizationResponseDto::from(organization))
    }
}
