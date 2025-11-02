use crate::application::dto::{GetOrganizationRequestDto, OrganizationResponseDto};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use std::sync::Arc;

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
    pub fn new(org_repo: Arc<R>) -> Self {
        Self { org_repo }
    }

    pub async fn execute(
        &self,
        request: GetOrganizationRequestDto,
    ) -> DomainResult<OrganizationResponseDto> {
        let organization = self.org_repo.find_by_id(&request.organization_id).await?;

        Ok(OrganizationResponseDto::from(organization))
    }
}
